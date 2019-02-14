//! The modifiers that exist in Sylan, and the set of modifiers that exist for each declaration
//! type.
//!
//! These sets are used for disambiguating parsing, amongst other things.

use std::collections::HashSet;

/// All declaration modifiers across Sylan. Modifiers are for compile-time constraints and parsing
/// disambiguation. Examples include access modifiers (e.g. public and internal), and the
/// overriding declarations.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Modifier {
    Accessibility,
    Virtual,
    Override,
    Ignorable,
    Embed,
    Extern,
}

pub fn new_package_modifiers() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.insert(Modifier::Accessibility);
    set
}

pub fn new_package_variable_modifiers() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.insert(Modifier::Accessibility);
    set.insert(Modifier::Extern);
    set
}

pub fn new_package_type_specification_modifiers() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.insert(Modifier::Accessibility);
    set
}

pub fn new_package_function_modifiers() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.insert(Modifier::Accessibility);
    set.insert(Modifier::Ignorable);
    set.insert(Modifier::Extern);
    set
}

pub fn new_method_modifiers() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.insert(Modifier::Accessibility);
    set.insert(Modifier::Virtual);
    set.insert(Modifier::Override);
    set.insert(Modifier::Ignorable);
    set.insert(Modifier::Extern);
    set
}

pub fn new_field_modifiers() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.insert(Modifier::Accessibility);
    set.insert(Modifier::Embed);
    set.insert(Modifier::Extern);
    set
}
