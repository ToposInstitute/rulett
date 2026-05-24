//! Rule-based models.

use super::{prelude::*, tm::*};

/// A declaration in the definition of a rule-based model.
pub enum ModelDecl {
    Agent {
        name: Name,
        interface: ObTm,
    },
    Rule {
        name: Option<Name>,
        interface: ObTm,
        lhs: Pattern,
        rhs: Pattern,
    },
}

/// Pattern in a rule-based model.
///
/// A pattern is represented as a restriction of a list of agents along a term.
pub struct Pattern {
    agents: Vec<Name>,
    term: MorTm,
}
