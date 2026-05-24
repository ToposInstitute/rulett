//! Generation of unique names.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::prelude::*;

/// Generate a unique [`Name`] with the given tag.
///
/// Follows the convention of Julia's `gensym`: the returned name has the form
/// `##<tag>#<n>`, where `n` is a monotonically increasing counter.
pub fn gensym(tag: &str) -> Name {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    Name::from(&format!("##{tag}#{n}"))
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
