//! Denormalize rustdoc output

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub)]

use std::io;

use rustdoc_types::{
    GenericBound, GenericParamDefKind, Term, TraitBoundModifier, Type, WherePredicate,
};

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
            functions: vec![],
        };

        for (path_name, module) in modules {
            let items = &module.items;
            output.parse_traits(&db, items, path_name);
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
        table::to_table(self)
    }

    fn parse_traits(&mut self, db: &Database, items: &[rustdoc_types::Id], path_name: String) {
        for (item, trait_) in db.find_traits(items) {
            let trait_name = item.name.unwrap();
            let decl = format_trait(&trait_name, &trait_);
            self.traits.push(Trait {
                name: trait_name,
                has_generics: trait_has_generics(&trait_),
                path: path_name.clone(),
                stability: parse_stability(&item.attrs),
                decl,
            });
            // TODO: find functions
        }
    }
}

/// A trait
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Trait {
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
}

/// An enum
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Enum {
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
}

/// A struct
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Struct {
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
}

/// A function
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Function {
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

/// What is the stability of this item?
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Stability {
    /// The item is stable
    Stable,
    /// The item is unstable
    Unstable,
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
    let mut val = Stability::Stable;
    for attr in attrs {
        if attr.starts_with("#[unstable") {
            val = Stability::Unstable;
        }
    }
    val
}
