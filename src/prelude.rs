//! Common imports for [`rulett`](crate).

pub use indexmap::{IndexMap, IndexSet};
pub use itertools::Itertools;
pub use std::collections::{HashMap, HashSet};

use ustr::Ustr;

/// Type of names in rulett.
pub type Name = Ustr;

/// Convenience function to coerce a string-like to a [`Name`].
pub fn name(s: impl Into<Name>) -> Name {
    s.into()
}
