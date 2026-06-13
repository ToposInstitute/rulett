//! Terms in core syntax.

use pretty::RcDoc;
use std::fmt;

use crate::prelude::*;

/// A segment of a path into the structure bound by a let binding.
///
/// A [bound variable](MorTm::BVar) refers to an enclosing let binding by its de
/// Bruijn index. Because a let binding may destructure a list or a tensor
/// rather than bind a single variable, the index alone does not identify which
/// component of the binding is meant; a sequence of segments forms a path to
/// the variable.
///
/// For example, in `let ⊗ [x, y] = t in ...` the variable `x` is reached by the
/// path `[Tensor, List(0)]` and `y` by `[Tensor, List(1)]`, whereas in
/// `let x = t in ...` the variable `x` is reached by the empty path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BVarSegment {
    List(usize),
    Tensor,
}

/// Morphism term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MorTm {
    /// A free variable.
    FVar(Name),

    /// A bound variable.
    BVar(usize, Vec<BVarSegment>),

    /// A list of terms.
    List(Vec<MorTm>),

    /// An application of the tensor product to a term.
    Tensor(Box<MorTm>),

    /// An application of an operation in the signature to a term.
    App(Name, Box<MorTm>),

    /// A let binding.
    Let { bound: Box<MorTm>, body: Box<MorTm> }
}

impl fmt::Display for MorTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        render_doc(self.to_doc(), f)
    }
}

impl FromIterator<MorTm> for MorTm {
    fn from_iter<T: IntoIterator<Item = MorTm>>(iter: T) -> Self {
        Self::List(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[MorTm; N]> for MorTm {
    fn from(value: [MorTm; N]) -> Self {
        Self::List(value.into())
    }
}

impl MorTm {
    /// Pretty document for the term.
    pub fn to_doc(&self) -> RcDoc<'static> {
        match self {
            MorTm::FVar(name) => RcDoc::text(name.as_str()),
            MorTm::BVar(index, path) => RcDoc::text(bvar_string(*index, path)),
            MorTm::List(terms) => bracketed("[", "]", terms.iter().map(MorTm::to_doc)),
            MorTm::Tensor(tm) => match &**tm {
                MorTm::List(terms) => bracketed("(", ")", terms.iter().map(MorTm::to_doc)),
                _ => RcDoc::text("⊗ ").append(tm.to_doc()),
            },
            MorTm::App(name, tm) => {
                RcDoc::text(name.as_str()).append(RcDoc::space()).append(tm.to_doc())
            }
            MorTm::Let { bound, body } => let_doc(bound.to_doc(), body.to_doc()),
        }
    }

    /// Smart constructor for [`FVar`](Self::FVar) variant.
    pub fn fvar(name: impl Into<Name>) -> Self {
        Self::FVar(name.into())
    }

    /// Smart constructor for [`BVar`](Self::BVar) variant.
    pub fn bvar(index: usize, path: impl IntoIterator<Item = BVarSegment>) -> Self {
        Self::BVar(index, path.into_iter().collect())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(terms: impl IntoIterator<Item = MorTm>) -> Self {
        Self::from_iter(terms)
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(tm: impl Into<MorTm>) -> Self {
        Self::Tensor(Box::new(tm.into()))
    }

    /// Smart constructor for [`App`](Self::App) variant.
    pub fn app(name: impl Into<Name>, tm: impl Into<MorTm>) -> Self {
        Self::App(name.into(), Box::new(tm.into()))
    }

    /// Smart constructor for [`Let`](Self::Let) variant.
    pub fn let_(bound: impl Into<MorTm>, body: impl Into<MorTm>) -> Self {
        Self::Let {
            bound: Box::new(bound.into()),
            body: Box::new(body.into()),
        }
    }
}

/// Renders a bound variable as a string.
///
/// The format is a dot-separated sequence of its de Bruijn index followed by
/// the positions of its [`List`](BVarSegment::List) segments.
fn bvar_string(index: usize, path: &[BVarSegment]) -> String {
    std::iter::once(index)
        .chain(path.iter().filter_map(|seg| match seg {
            BVarSegment::List(i) => Some(*i),
            BVarSegment::Tensor => None,
        }))
        .join(".")
}

/// Pattern term.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatTm {
    /// A restriction of an agent or a basic rule along a morphism.
    Res(Name, MorTm),

    /// A list of terms.
    List(Vec<PatTm>),

    /// An application of the tensor product.
    Tensor(Box<PatTm>),

    /// A let binding.
    Let { bound: MorTm, body: Box<PatTm> }
}

impl fmt::Display for PatTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        render_doc(self.to_doc(), f)
    }
}

impl FromIterator<PatTm> for PatTm {
    fn from_iter<T: IntoIterator<Item = PatTm>>(iter: T) -> Self {
        Self::List(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[PatTm; N]> for PatTm {
    fn from(value: [PatTm; N]) -> Self {
        Self::List(value.into())
    }
}

impl PatTm {
    /// Pretty document for the pattern term.
    pub fn to_doc(&self) -> RcDoc<'static> {
        match self {
            PatTm::Res(name, tm) => {
                RcDoc::text(name.as_str()).append(RcDoc::space()).append(tm.to_doc())
            }
            PatTm::List(patterns) => bracketed("[", "]", patterns.iter().map(PatTm::to_doc)),
            PatTm::Tensor(tm) => match &**tm {
                PatTm::List(terms) => bracketed("(", ")", terms.iter().map(PatTm::to_doc)),
                _ => RcDoc::text("⊗ ").append(tm.to_doc()),
            },
            PatTm::Let { bound, body } => let_doc(bound.to_doc(), body.to_doc()),
        }
    }

    /// Smart constructor for [`Res`](Self::Res) variant.
    pub fn res(name: impl Into<Name>, tm: impl Into<MorTm>) -> Self {
        Self::Res(name.into(), tm.into())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(patterns: impl IntoIterator<Item = PatTm>) -> Self {
        Self::from_iter(patterns)
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(pattern: impl Into<PatTm>) -> Self {
        Self::Tensor(Box::new(pattern.into()))
    }

    /// Smart constructor for [`Let`](Self::Let) variant.
    pub fn let_(bound: impl Into<MorTm>, body: impl Into<PatTm>) -> Self {
        Self::Let {
            bound: bound.into(),
            body: Box::new(body.into()),
        }
    }
}

/// Pretty document for `let <bound> in <body>`, breakable after `in`.
fn let_doc<'a>(bound: RcDoc<'a>, body: RcDoc<'a>) -> RcDoc<'a> {
    RcDoc::text("let ")
        .append(bound)
        .append(RcDoc::text(" in"))
        .append(RcDoc::line().append(body).nest(2))
        .group()
}
