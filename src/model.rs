//! Rule-based models.

use super::{prelude::*, theory::*};

/// A declaration in the definition of a rule-based model.
pub enum ModelDecl {
    Agent {
        name: Name,
        interface: ObTerm,
    },
    Rule {
        name: Option<Name>,
        interface: ObTerm,
        lhs: Pattern,
        rhs: Pattern,
    },
}

/// Pattern in a rule-based model.
///
/// A pattern is represented as a restriction of a list of agents along a term.
pub struct Pattern {
    agents: Vec<Name>,
    term: MorTerm,
}
