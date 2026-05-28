//! Names (interned symbols).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use ustr::Ustr;

/// Type of names (internal symbols) in this crate.
pub type Name = Ustr;

/// Convenience function to coerce a string-like to a [`Name`].
pub fn name(s: impl Into<Name>) -> Name {
    s.into()
}

/// Generate a unique [`Name`] with the given tag.
///
/// Follows the convention of Julia's `gensym`: the returned name has the form
/// `##<tag>#<n>`, where `n` is a monotonically increasing counter.
pub fn gensym(tag: &str) -> Name {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    Name::from(&format!("##{tag}#{n}"))
}

/// Ensure that all names are unique by appending numbers if necessary.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gensym_is_unique() {
        let a = gensym("x");
        let b = gensym("x");
        assert_ne!(a, b);
        assert!(a.as_str().starts_with("##x#"));
        assert!(b.as_str().starts_with("##x#"));
    }
}
