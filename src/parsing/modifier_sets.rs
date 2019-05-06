//! Certain items (as opposed to expressions) can take a set of modifiers. These can control
//! accessibility, purity, and more.
//!
//! Modifiers always appear after a disambiguating keyword, unlike many other languages that put
//! them before and expect the human reader to wade through a handful of words before working out
//! what they're actually modifying.
//!
//! This module tracks all possible modifier sets. Where modifiers are allowed, the context works
//! out which modifier set to look at. The parser then chews through a subset of modifiers in the
//! set for that context.
//!
//! TODO: reevaluate the purity modifiers once effect-tracking is investigated more thoroughly.

use lexing::tokens::Token::{
    self, Embed, Extern, Ignorable, Internal, Operator, Override, Public, Virtual,
};
use std::collections::HashSet;

pub fn new_package_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![Public, Internal]);
    set
}

pub fn new_interface_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![Public, Internal]);
    set
}

pub fn new_class_and_enum_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![Public, Internal, Extern]);
    set
}

pub fn new_function_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![Public, Internal, Ignorable, Extern, Operator]);
    set
}

pub fn new_method_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![
        Public, Internal, Ignorable, Virtual, Override, Extern, Operator,
    ]);
    set
}

/// As `func` is just syntactical sugar for `var` with a lambda expression, all of the modifiers
/// for `func` also work on `var`, but might be rejected later on in the compiler if the type of
/// the `var` binding doesn't resolve to a function type.
pub fn new_var_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![
        Public, Internal, Ignorable, Virtual, Override, Embed, Extern, Operator,
    ]);
    set
}

pub fn new_constructor_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![Public, Internal, Virtual, Override, Extern]);
    set
}

pub fn new_class_extension_modifier_set() -> HashSet<Token> {
    let mut set = HashSet::new();
    set.extend(vec![Public, Internal]);
    set
}
