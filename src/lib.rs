//! Denormalize rustdoc output

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub)]

use std::io;

use cli_table::TableStruct;
use rustdoc_types::{
    GenericBound, GenericParamDefKind, Term, TraitBoundModifier, Type, WherePredicate,
};
use serde::{Deserialize, Serialize};

mod database;
mod table;
use database::Database;

/// A crate
#[derive(Debug, PartialEq, PartialOrd, Default)]
pub struct Crate {
    /// Traits contained in this crate
    pub traits: Vec<Trait>,
    /// Structs contained in this crate
    pub structs: Vec<Struct>,
    /// Enums contained in this crate
    pub enums: Vec<Enum>,
    /// Impls contained in this crate
    pub impls: Vec<Impl>,
    /// Functions and methods contained in this crate
    pub functions: Vec<Function>,
}

impl Crate {
    /// Create a new instance from a string slice.
    pub fn from_str(s: &str) -> io::Result<Self> {
        let krate: rustdoc_types::Crate = serde_json::from_str(&s)?;
        let db = Database::new(krate);
        let modules = db.modules();

        let mut output = Self {
            traits: vec![],
            structs: vec![],
            enums: vec![],
            impls: vec![],
            functions: vec![],
        };

        for (path_name, module) in modules {
            let items = &module.items;
            output.parse_traits(&db, items, &path_name);
            output.count_functions(&db, items, &path_name, false);
            output.parse_structs(&db, items, &path_name);
            output.parse_enums(&db, items, &path_name);
        }

        output.traits.sort();
        output.traits.dedup();
        output.structs.sort();
        output.structs.dedup();
        output.enums.sort();
        output.enums.dedup();
        output.impls.sort();
        output.impls.dedup();
        output.functions.sort();
        output.functions.dedup();

        Ok(output)
    }

    /// Move all items from `other` into `self` leaving `other` empty
    pub fn append(&mut self, other: &mut Self) {
        self.traits.append(&mut other.traits);
        self.structs.append(&mut other.structs);
        self.enums.append(&mut other.enums);
        self.functions.append(&mut other.functions);
    }

    /// Output the contents of the crate as a table
    pub fn to_table(&self) -> TableStruct {
        table::to_table(self)
    }

    fn parse_traits(&mut self, db: &Database, items: &[rustdoc_types::Id], path_name: &str) {
        for (item, trait_) in db.find_traits(items) {
            let trait_name = item.name.unwrap();
            let decl = format_trait(&trait_name, &trait_);
            let has_generics = contains_generics(&trait_.generics);

            let fn_path = format!("{path_name}::{}", &trait_name);
            let fn_count = self.count_functions(db, &trait_.items, &fn_path, has_generics);

            let stability = parse_stability(&item.attrs);

            self.traits.push(Trait {
                kind: "trait",
                name: trait_name.clone(),
                has_generics,
                path: path_name.to_string(),
                stability,
                fn_count,
                decl,
            });
        }
    }

    fn parse_structs(&mut self, db: &Database, items: &[rustdoc_types::Id], path_name: &str) {
        // dbg!(items.contains(&Id(String::from("0:3663:9709"))));
        for (item, strukt) in db.find_structs(items) {
            let strukt_name = item.name.unwrap();
            // println!("{strukt_name}");
            let decl = format_struct(&strukt_name, &strukt);
            let has_generics = contains_generics(&strukt.generics);

            let strukt_path = format!("{path_name}::{}", &strukt_name);
            let fn_count = self.count_inherent_impls(db, &strukt.impls, &strukt_path);

            let stability = parse_stability(&item.attrs);
            self.parse_trait_impls(db, &strukt.impls, path_name, stability);

            self.structs.push(Struct {
                kind: "struct",
                name: strukt_name.clone(),
                has_generics,
                path: path_name.to_string(),
                stability: parse_stability(&item.attrs),
                fn_count,
                decl,
            });
        }
    }

    fn parse_enums(&mut self, db: &Database, items: &[rustdoc_types::Id], path_name: &str) {
        for (item, enum_) in db.find_enums(items) {
            let trait_name = item.name.unwrap();
            let decl = format_enum(&trait_name, &enum_);

            let enum_path = format!("{path_name}::{}", &trait_name);
            let fn_count = self.count_inherent_impls(db, &enum_.impls, &enum_path);
            let stability = parse_stability(&item.attrs);
            self.parse_trait_impls(db, &enum_.impls, path_name, stability);

            self.enums.push(Enum {
                kind: "enum",
                name: trait_name.clone(),
                has_generics: contains_generics(&enum_.generics),
                path: path_name.to_string(),
                stability,
                fn_count,
                decl,
            });
        }
    }

    fn parse_trait_impls(
        &mut self,
        db: &Database,
        items: &[rustdoc_types::Id],
        path_name: &str,
        mut stability: Stability,
    ) {
        for (_item, impl_) in db.find_impls(items) {
            let has_generics = contains_generics(&impl_.generics);

            // We're only interested in trait impls
            if let Some(trait_) = impl_.trait_.clone() {
                db.find_enums(&impl_.items)
                    .into_iter()
                    .for_each(|(item, _)| {
                        if let Stability::Unstable = parse_stability(&item.attrs) {
                            stability = Stability::Unstable;
                        }
                    });
                match db.find_traits(&[trait_.id]).into_iter().next() {
                    Some((trait_item, _)) => {
                        if let Stability::Unstable = parse_stability(&trait_item.attrs) {
                            stability = Stability::Unstable;
                        }
                    }
                    // Assume stable stability if it's an external trait
                    None => {}
                }

                // NOTE: The bug here is that the item is in a separate crate!
                // External traits can be implemented in this crate.

                // TODO: we should just do a name-based lookup for traits here?
                // TODO: this requires processing crates per section, not per crate

                let decl = format_impl(impl_);
                self.impls.push(Impl {
                    kind: "impl",
                    name: trait_.name.clone(),
                    has_generics,
                    path: path_name.to_string(),
                    stability,
                    fn_count: 0,
                    decl,
                });
            }
        }
    }

    fn count_inherent_impls(
        &mut self,
        db: &Database,
        items: &[rustdoc_types::Id],
        path_name: &str,
    ) -> usize {
        let mut count = 0;
        for (_item, impl_) in db.find_impls(items) {
            // We're only interested in inherent impls
            if impl_.trait_.is_some() || impl_.synthetic || impl_.blanket_impl.is_some() {
                continue;
            }
            let has_generics = contains_generics(&impl_.generics);
            count += self.count_functions(db, &impl_.items, &path_name, has_generics);
        }
        count
    }

    fn count_functions(
        &mut self,
        db: &Database,
        items: &[rustdoc_types::Id],
        path_name: &str,
        parent_has_generics: bool,
    ) -> usize {
        let mut count = 0;
        for (item, fn_) in db.find_functions(&items) {
            count += 1;
            let function_name = item.name.unwrap();
            self.functions.push(Function {
                kind: "function",
                name: function_name.clone(),
                has_generics: contains_generics(&fn_.generics) || parent_has_generics,
                path: path_name.to_owned(),
                stability: parse_stability(&item.attrs),
                decl: format_function(&function_name, &fn_),
                fn_count: 0,
            });
        }
        count
    }
}

/// A trait
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct Trait {
    /// What kind of item is this?
    pub kind: &'static str,
    /// The name
    pub name: String,
    /// The path without the name
    pub path: String,
    /// The signature of the item
    pub decl: String,
    /// Does this item have generics?
    pub has_generics: bool,
    /// What is the stability of this item?
    pub stability: Stability,
    /// How many methods does this item have?
    pub fn_count: usize,
}

/// An enum
#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Ord, Eq)]
pub struct Enum {
    /// What kind of item is this?
    pub kind: &'static str,
    /// The name
    pub name: String,
    /// The path without the name
    pub path: String,
    /// The signature of the item
    pub decl: String,
    /// Does this item have generics?
    pub has_generics: bool,
    /// What is the stability of this item?
    pub stability: Stability,
    /// How many methods does this item have?
    pub fn_count: usize,
}

/// A struct
#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Ord, Eq)]
pub struct Struct {
    /// What kind of item is this?
    pub kind: &'static str,
    /// The name
    pub name: String,
    /// The path without the name
    pub path: String,
    /// The signature of the item
    pub decl: String,
    /// Does this item have generics?
    pub has_generics: bool,
    /// What is the stability of this item?
    pub stability: Stability,
    /// How many methods does this item have?
    pub fn_count: usize,
}

/// A function
#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Ord, Eq)]
pub struct Function {
    /// What kind of item is this?
    pub kind: &'static str,
    /// The name
    pub name: String,
    /// The path without the name
    pub path: String,
    /// The signature of the item
    pub decl: String,
    /// Does this item have generics?
    pub has_generics: bool,
    /// What is the stability of this item?
    pub stability: Stability,
    /// How many methods does this item have?
    pub fn_count: usize,
}

/// A struct
#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize, Ord, Eq)]
pub struct Impl {
    /// What kind of item is this?
    pub kind: &'static str,
    /// The name
    pub name: String,
    /// The path without the name
    pub path: String,
    /// The signature of the item
    pub decl: String,
    /// Does this item have generics?
    pub has_generics: bool,
    /// What is the stability of this item?
    pub stability: Stability,
    /// How many methods does this item have?
    pub fn_count: usize,
}

fn contains_generics(generics: &rustdoc_types::Generics) -> bool {
    let params = &generics
        .params
        .iter()
        .filter(|p| !matches!(p.kind, GenericParamDefKind::Lifetime { .. }))
        .count();

    let wheres = &generics
        .where_predicates
        .iter()
        .filter(|p| matches!(p, WherePredicate::BoundPredicate { .. }))
        .count();
    (params + wheres) != 0
}

fn format_function(name: &str, fn_: &rustdoc_types::Function) -> String {
    if name == "merge_sort" {
        return format!("<merge sort is unstable and annoyingly complicated>");
    }
    let is_const = if fn_.header.const_ { "const " } else { "" };
    let is_unsafe = if fn_.header.const_ { "unsafe " } else { "" };
    let is_async = if fn_.header.async_ { "async " } else { "" };
    let body = if fn_.has_body { " { .. }" } else { ";" };
    let output = match &fn_.decl.output {
        Some(ty) => format!(" -> {}", format_type(&ty)),
        None => String::new(),
    };
    let args = &fn_
        .decl
        .inputs
        .iter()
        .map(|(name, ty)| format!("{name}: {}", format_type(ty)))
        .collect::<Vec<_>>();
    let args = args.join(", ");
    let params = format_generic_params(&fn_.generics.params);
    let where_bounds = format_where_bounds(&fn_.generics.where_predicates);
    format!("{is_const}{is_unsafe}{is_async}fn {name}{params}({args}){output}{where_bounds}{body}")
}

fn format_trait(name: &str, trait_: &rustdoc_types::Trait) -> String {
    let is_auto = if trait_.is_auto { "auto " } else { "" };
    let is_unsafe = if trait_.is_unsafe { "unsafe " } else { "" };
    let params = format_generic_params(&trait_.generics.params);
    let where_bounds = format_where_bounds(&trait_.generics.where_predicates);
    let trait_bounds = format_generic_bounds(&trait_.bounds);
    format!("{is_unsafe}{is_auto}trait {name}{params}{trait_bounds} {where_bounds}{{ }}")
}

fn format_struct(name: &str, strukt: &rustdoc_types::Struct) -> String {
    let params = format_generic_params(&strukt.generics.params);
    let where_bounds = format_where_bounds(&strukt.generics.where_predicates);
    format!("struct {name}{params} {where_bounds} {{ .. }}")
}

fn format_enum(name: &str, strukt: &rustdoc_types::Enum) -> String {
    let params = format_generic_params(&strukt.generics.params);
    let where_bounds = format_where_bounds(&strukt.generics.where_predicates);
    format!("enum {name}{params} {where_bounds} {{ .. }}")
}

fn format_generic_params(params: &[rustdoc_types::GenericParamDef]) -> String {
    let mut out = vec![];
    for param in params {
        let name = &param.name;
        match &param.kind {
            GenericParamDefKind::Lifetime { outlives: _ } => continue,
            GenericParamDefKind::Type {
                bounds,
                default,
                synthetic: _,
            } => {
                // if *synthetic {
                //     continue;
                // }
                let bounds = format_generic_bounds(&bounds);
                let default = match default {
                    Some(ty) => format!(" = {}", format_type(ty)),
                    None => String::new(),
                };
                out.push(format!("{name}{bounds}{default}"))
            }
            GenericParamDefKind::Const { type_, default } => match default {
                Some(default) => out.push(format!("const {name}: {type_:?} = {default}")),
                None => out.push(format!("const {name}: {type_:?}")),
            },
        }
    }
    match out.len() {
        0 => String::new(),
        _ => format!("<{}>", out.join(", ")),
    }
}

fn format_generic_bounds(bounds: &[GenericBound]) -> String {
    let mut out = vec![];
    for bound in bounds {
        match &bound {
            GenericBound::TraitBound {
                trait_,
                generic_params: _, // TODO: support HRTBs
                modifier,
            } => {
                let trait_ = &trait_.name;
                let modifier = match modifier {
                    TraitBoundModifier::None => "",
                    TraitBoundModifier::Maybe => "?",
                    TraitBoundModifier::MaybeConst => "~const ",
                };
                out.push(format!("{modifier}{trait_}"));
            }
            GenericBound::Outlives(_) => continue, // TODO: support lifetimes
        };
    }
    match out.len() {
        0 => String::new(),
        _ => format!(": {}", out.join(" + ")),
    }
}

fn format_where_bounds(predicates: &[WherePredicate]) -> String {
    let mut out = vec![];
    for pred in predicates {
        match pred {
            WherePredicate::BoundPredicate {
                type_,
                bounds,
                generic_params: _, // TODO: HRTBs
            } => out.push(format!(
                "{}{}",
                format_type(type_),
                format_generic_bounds(bounds)
            )),
            WherePredicate::RegionPredicate {
                lifetime: _,
                bounds: _,
            } => out.push(format!("todo: region predicate")),
            WherePredicate::EqPredicate { lhs, rhs } => {
                out.push(format!("{} = {}", format_type(lhs), format_term(rhs)))
            }
        }
    }
    match out.len() {
        0 => String::new(),
        _ => format!(" where {}", out.join(", ")),
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Generic(generic) => generic.clone(),
        Type::QualifiedPath {
            name,
            args: _, // TODO: unsure what this is
            self_type,
            trait_: _, // TODO: I believe this is `<x as trait_>` bounds?
        } => {
            format!("{}::{name}", format_type(self_type))
        }
        Type::BorrowedRef {
            lifetime,
            mutable,
            type_,
        } => {
            let lifetime = match lifetime {
                Some(lt) => lt.clone(),
                None => String::new(),
            };
            let mutable = if *mutable { " mut" } else { "" };
            format!("&{lifetime}{mutable} {}", format_type(type_))
        }
        Type::Primitive(ty) => ty.to_owned(),
        Type::ResolvedPath(path) => path.name.clone(),
        Type::Tuple(data) => {
            let output: Vec<_> = data.iter().map(|ty| format_type(ty)).collect();
            output.join(", ")
        }
        Type::Slice(ty) => format_type(ty),
        Type::RawPointer { mutable, type_ } => match mutable {
            true => format!("*mut {}", format_type(type_)),
            false => format!("*const {}", format_type(type_)),
        },
        Type::FunctionPointer(_ptr) => format!("<todo: fn pointer>"),
        Type::DynTrait(dyn_trait) => {
            let traits: Vec<_> = dyn_trait
                .traits
                .iter()
                .map(|t| t.trait_.name.clone())
                .collect();
            format!("dyn {}", traits.join(" + "))
        }
        Type::ImplTrait(bounds) => format!("impl {}", format_generic_bounds(&bounds)),
        Type::Array { type_, len } => format!("[{}; {len}]", format_type(type_)),
        ty => format!("todo format type: {ty:?}>"),
    }
}

fn format_impl(impl_: rustdoc_types::Impl) -> String {
    let is_unsafe = match impl_.is_unsafe {
        true => "",
        false => "unsafe ",
    };
    let trait_ = match impl_.trait_ {
        Some(trait_) => format!("{} for ", trait_.name),
        None => String::new(),
    };
    let ty = format_type(&impl_.for_);
    let params = format_generic_params(&impl_.generics.params);
    let where_bounds = format_where_bounds(&impl_.generics.where_predicates);
    format!("{is_unsafe} impl{params} {trait_} {ty} {where_bounds} {{}}")
}

fn format_term(term: &Term) -> String {
    match term {
        Term::Type(ty) => format_type(ty),
        Term::Constant(c) => format_constant(c),
    }
}

fn format_constant(_c: &rustdoc_types::Constant) -> String {
    format!("todo: format constants")
}

/// What is the stability of this item?
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
pub enum Stability {
    /// The item is stable
    Stable,
    /// The item is unstable
    Unstable,
}
impl Stability {
    /// Returns `true` if the stability is [`Stable`].
    ///
    /// [`Stable`]: Stability::Stable
    #[must_use]
    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Stable)
    }

    /// Returns `true` if the stability is [`Unstable`].
    ///
    /// [`Unstable`]: Stability::Unstable
    #[must_use]
    pub fn is_unstable(&self) -> bool {
        matches!(self, Self::Unstable)
    }
}

impl std::fmt::Display for Stability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Unstable => write!(f, "unstable"),
        }
    }
}

fn parse_stability(attrs: &[String]) -> Stability {
    let mut val = Stability::Unstable;
    for attr in attrs {
        if attr.contains("#[stable") {
            val = Stability::Stable;
        }
    }
    val
}
