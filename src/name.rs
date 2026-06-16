//! Names (interned symbols).

use std::collections::HashMap;
use ustr::Ustr;

/// Type of names (internal symbols) in this crate.
pub type Name = Ustr;

/// Convenience function to coerce a string-like to a [`Name`].
pub fn name(s: impl Into<Name>) -> Name {
    s.into()
}

/// Ensure that all names are unique by appending numbers if necessary.
///
/// Assumes that no name ends with `#{n}` for some `n`.
pub fn uniquify_names(names: &mut [Name]) {
    let mut counts = HashMap::new();
    for name in names.iter().copied() {
        *counts.entry(name).or_insert(0) += 1;
    }

    counts.retain(|_, count| *count > 1);
    for name in names.iter_mut().rev() {
        if let Some(count) = counts.get_mut(name) {
            *name = format!("{}#{}", name, count).into();
            *count -= 1;
        }
    }
}
