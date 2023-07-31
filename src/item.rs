use super::Stability;
use serde::{Deserialize, Serialize};

/// A trait
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct Item {
    /// What kind of item is this?
    pub kind: &'static str,
    /// The rustdoc ID assigned to this item
    pub id: String,
    /// The name
    pub name: String,
    /// The path without the name
    pub path: String,
    /// The signature of the item
    pub decl: String,
    /// Does this item have generics?
    pub has_generics: bool,
    /// Is this a const item?
    pub is_const: bool,
    /// Is this an async item?
    pub is_async: bool,
    /// What is the stability of this item?
    pub stability: Stability,
    /// How many methods does this item have?
    pub fn_count: usize,
}
