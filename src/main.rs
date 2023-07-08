//! Analyze the items in the stdlib
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub)]

use std::{fs, io};

use rustdoc_types::{
    GenericBound, GenericParamDefKind, ItemEnum, Term, TraitBoundModifier, Type, WherePredicate,
};

fn main() -> io::Result<()> {
    let mut core = Crate::from_str(&fs::read_to_string("assets/core.json")?)?;
    let mut alloc = Crate::from_str(&fs::read_to_string("assets/alloc.json")?)?;
    let mut std = Crate::from_str(&fs::read_to_string("assets/std.json")?)?;

    core.append(&mut alloc);
    core.append(&mut std);

    println!("{}", core.to_table());
    Ok(())
}

/// A crate
#[derive(Debug, PartialEq, PartialOrd, Default)]
pub struct Crate {
    traits: Vec<Trait>,
    structs: Vec<Struct>,
    enums: Vec<Enum>,
    functions: Vec<Function>,
}
impl Crate {
    /// Create a new instance from a string slice.
    pub fn from_str(s: &str) -> io::Result<Self> {
        let krate: rustdoc_types::Crate = serde_json::from_str(&s)?;
        let db = Database::new(krate);
        let modules = db.modules();

        let mut output = Crate::default();
        for (path_name, module) in modules {
            // Find traits
            for (trait_name, trait_) in db.find_traits(&module.items) {
                let decl = format_trait(&trait_name, &trait_);
                output.traits.push(Trait {
                    name: trait_name,
                    has_generics: trait_has_generics(&trait_),
                    path: path_name.clone(),
                    decl,
                });
            }
        }
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
    pub fn to_table(&self) -> String {
        use cli_table::{Cell, Style, Table};
        let output = self
            .traits
            .iter()
            .map(|t| {
                vec![
                    "trait".cell(),
                    format!("{}::{}", t.path, t.name).cell(),
                    t.decl.clone().cell(),
                    t.has_generics.cell(),
                ]
            })
            .collect::<Vec<_>>();
        output
            .table()
            .title(vec![
                "Kind".cell().bold(true),
                "Name".cell().bold(true),
                "Signature".cell().bold(true),
                "Generics?".cell().bold(true),
            ])
            .display()
            .unwrap()
            .to_string()
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Trait {
    name: String,
    path: String,
    decl: String,
    has_generics: bool,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Enum {
    name: String,
    path: String,
    decl: String,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Struct {
    name: String,
    path: String,
    decl: String,
}

#[derive(Debug, PartialEq, PartialOrd)]
struct Function {
    name: String,
    path: String,
    decl: String,
}

/// Internal rustdoc database structure with various query methods on it.
struct Database {
    inner: rustdoc_types::Crate,
}

impl Database {
    /// Create a new instance of database
    fn new(inner: rustdoc_types::Crate) -> Self {
        Self { inner }
    }

    /// Find a rustdoc `Item` by id
    fn find_item(&self, id: &rustdoc_types::Id) -> Option<rustdoc_types::Item> {
        let item = self.inner.index.get(id)?;
        Some(item.clone())
    }

    /// Find a rustdoc path by id.
    fn find_path(&self, id: &rustdoc_types::Id) -> Option<String> {
        let summary = self.inner.paths.get(id)?;
        Some(summary.path.join("::"))
    }

    /// Get a list of all modules
    fn modules(&self) -> Vec<(String, rustdoc_types::Module)> {
        let mut out: Vec<_> = self
            .inner
            .index
            .iter()
            .filter_map(|(id, item)| match &item.inner {
                ItemEnum::Module(module) => {
                    if module.is_stripped {
                        return None;
                    }
                    let path = self.find_path(id)?;
                    Some((path, module.clone()))
                }
                _ => None,
            })
            .collect();
        out.sort_by(|(left, _), (right, _)| left.cmp(right));
        out
    }

    /// Given a list of IDs, find all traits. A rustdoc module only
    /// provides a `Vec<Id>` for all items in it, so we have to do a filter-find
    /// to narrow it down to just traits, etc.
    fn find_traits(&self, ids: &[rustdoc_types::Id]) -> Vec<(String, rustdoc_types::Trait)> {
        ids.into_iter()
            .filter_map(|id| {
                self.find_item(id).and_then(|item| match item.inner {
                    ItemEnum::Trait(adt) => Some((item.name.unwrap(), adt)),
                    _ => None,
                })
            })
            .collect()
    }
}

fn trait_has_generics(trait_: &rustdoc_types::Trait) -> bool {
    let params = &trait_
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p.kind, GenericParamDefKind::Lifetime { .. }))
        .count();

    let wheres = &trait_
        .generics
        .where_predicates
        .iter()
        .filter(|p| matches!(p, WherePredicate::BoundPredicate { .. }))
        .count();
    (params + wheres) != 0
}

fn format_trait(name: &str, trait_: &rustdoc_types::Trait) -> String {
    let is_auto = if trait_.is_auto { "auto " } else { "" };
    let is_unsafe = if trait_.is_unsafe { "unsafe " } else { "" };
    let params = format_generic_params(&trait_.generics.params);
    let where_bounds = format_where_bounds(&trait_.generics.where_predicates);
    let trait_bounds = format_generic_bounds(&trait_.bounds);
    format!("{is_unsafe}{is_auto}trait {name}{params}{trait_bounds} {where_bounds}{{ }}")
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
                synthetic,
            } => {
                if *synthetic {
                    continue;
                }
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
                "{}: {}",
                format_type(type_),
                format_generic_bounds(bounds)
            )),
            WherePredicate::RegionPredicate {
                lifetime: _,
                bounds: _,
            } => todo!(), // TODO: lifetimes
            WherePredicate::EqPredicate { lhs, rhs } => {
                out.push(format!("{} = {}", format_type(lhs), format_term(rhs)))
            }
        }
    }
    match out.len() {
        0 => String::new(),
        _ => format!("where {}", out.join(", ")),
    }
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Generic(generic) => generic.clone(),
        ty => format!("<cannot format type: {ty:?}>"),
    }
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
