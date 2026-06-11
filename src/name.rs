//! Names (interned symbols).

use imbl as im;
use std::collections::HashMap;
use ustr::Ustr;

/// Type of names (internal symbols) in this crate.
pub type Name = Ustr;

/// Convenience function to coerce a string-like to a [`Name`].
pub fn name(s: impl Into<Name>) -> Name {
    s.into()
}

/// Generates unique names (across calls to this object).
///
/// Uses a persistent data structure, so is cheap to clone.
#[derive(Clone, Default)]
pub struct NameGenerator(im::HashMap<Name, usize>);

impl NameGenerator {
    /// Generate a unique [`Name`] with the given tag.
    ///
    /// Follows the convention of Julia's `gensym`: the returned name has the
    /// form `##<tag>#<n>`, where `n` is a monotonically increasing counter.
    pub fn gensym(&mut self, tag: &str) -> Name {
        let n = self.0.entry(tag.into()).or_default();
        *n += 1;
        Name::from(&format!("##{tag}#{n}"))
    }
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
