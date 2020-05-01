//! Certain items (as opposed to expressions) can take a set of modifiers. These can control
//! accessibility, purity, and more.
//!
//! Modifiers always appear after a disambiguating keyword, unlike many other languages that put
//! them before and expect the human reader to wade through a handful of words before working out
//! what they're actually modifying.
//! This module tracks all possible modifier sets. Where modifiers are allowed, the context works
//! out which modifier set to look at. The parser then chews through a subset of modifiers in the
//! set for that context.
//!
//! TODO: reevaluate the purity modifiers once effect-tracking is investigated more thoroughly.

use crate::common::multiphase::Accessibility;
use crate::lexing::tokens::Modifier::{self, Embed, Ignorable, Operator, Override};
use std::collections::{HashMap, HashSet};

pub struct ModifierSets {
    pub package: HashSet<Modifier>,
    pub interface: HashSet<Modifier>,
    pub class_and_enum: HashSet<Modifier>,
    pub function: HashSet<Modifier>,
    pub method: HashSet<Modifier>,
    pub binding: HashSet<Modifier>,
    pub field: HashSet<Modifier>,
    pub class_extension: HashSet<Modifier>,
}

pub struct AccessibilityModifierExtractor {
    accessibility_tokens: HashMap<Modifier, Accessibility>,
}

impl AccessibilityModifierExtractor {
    pub fn new() -> Self {
        let mut accessibility_tokens = HashMap::new();
        accessibility_tokens.insert(
            Modifier::Accessibility(Accessibility::Private),
            Accessibility::Public,
        );
        accessibility_tokens.insert(
            Modifier::Accessibility(Accessibility::Private),
            Accessibility::Internal,
        );
        Self {
            accessibility_tokens,
        }
    }

    pub fn extract_accessibility_modifier(
        &self,
        modifiers: &HashSet<Modifier>,
    ) -> Result<Accessibility, String> {
        let accessibility_modifiers: Vec<Modifier> = modifiers
            .iter()
            .filter(|modifier| self.accessibility_tokens.contains_key(&modifier))
            .cloned()
            .collect();

        match accessibility_modifiers.len() {
            0 => Ok(Accessibility::Private),
            1 => Ok(self.accessibility_tokens[&accessibility_modifiers[0]].clone()),
            _ => Err(format!(
                "multiple accessibility modifiers: {:?}",
                accessibility_modifiers
            )),
        }
    }
}

impl Default for ModifierSets {
    fn default() -> ModifierSets {
        ModifierSets {
            package: new_package_modifier_set(),
            interface: new_interface_modifier_set(),
            class_and_enum: new_class_and_enum_modifier_set(),
            function: new_function_modifier_set(),
            method: new_method_modifier_set(),
            binding: new_binding_modifier_set(),
            field: new_field_modifier_set(),
            class_extension: new_class_extension_modifier_set(),
        }
    }
}

fn new_package_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
    ]);
    set
}

fn new_interface_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
    ]);
    set
}

fn new_class_and_enum_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
    ]);
    set
}

fn new_function_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
        Ignorable,
        Operator,
    ]);
    set
}

fn new_method_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
        Ignorable,
        Override,
        Operator,
    ]);
    set
}

fn new_binding_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
    ]);
    set
}

fn new_field_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
        Embed,
    ]);
    set
}

fn new_class_extension_modifier_set() -> HashSet<Modifier> {
    let mut set = HashSet::new();
    set.extend(vec![
        Modifier::Accessibility(Accessibility::Public),
        Modifier::Accessibility(Accessibility::Internal),
    ]);
    set
}
