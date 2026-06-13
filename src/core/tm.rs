//! Terms in core syntax.

use pretty::RcDoc;
use std::fmt;

use crate::{ob_tm::*, prelude::*};

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
    Let { bound: Box<MorTm>, body: Box<MorTm> },
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

    /// Simultaneously substitutes terms for free variables in the term.
    ///
    /// Capture-avoiding due to the locally nameless representation.
    pub fn subst(&self, subst: &[(Name, MorTm)]) -> Self {
        match self {
            MorTm::FVar(name) => subst
                .iter()
                .rev()
                .find_map(|(n, tm)| (n == name).then(|| tm.clone()))
                .unwrap_or_else(|| self.clone()),
            MorTm::BVar(..) => self.clone(),
            MorTm::List(terms) => MorTm::list(terms.iter().map(|t| t.subst(subst))),
            MorTm::Tensor(tm) => MorTm::tensor(tm.subst(subst)),
            MorTm::App(name, tm) => MorTm::app(*name, tm.subst(subst)),
            MorTm::Let { bound, body } => MorTm::let_(bound.subst(subst), body.subst(subst)),
        }
    }

    /// Binds free variables with a `let in` expression.
    pub fn bind(self, bindings: &ObTm, bound: MorTm) -> Self {
        MorTm::let_(bound, self.close(bindings))
    }

    /// Closes over free variables by turning them into bound variables.
    pub fn close(self, bindings: &ObTm) -> Self {
        self.close_rec(&binding_paths(bindings), 0)
    }

    fn close_rec(self, paths: &IndexMap<Name, Vec<BVarSegment>>, depth: usize) -> Self {
        match self {
            MorTm::FVar(name) => match paths.get(&name) {
                Some(path) => MorTm::BVar(depth, path.clone()),
                None => MorTm::FVar(name),
            },
            bvar @ MorTm::BVar(..) => bvar,
            MorTm::List(terms) => MorTm::list(terms.into_iter().map(|t| t.close_rec(paths, depth))),
            MorTm::Tensor(tm) => MorTm::tensor(tm.close_rec(paths, depth)),
            MorTm::App(name, tm) => MorTm::app(name, tm.close_rec(paths, depth)),
            MorTm::Let { bound, body } => {
                MorTm::let_(bound.close_rec(paths, depth), body.close_rec(paths, depth + 1))
            }
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

/// Maps each variable in a binding structure to the path that reaches it.
fn binding_paths(bindings: &ObTm) -> IndexMap<Name, Vec<BVarSegment>> {
    fn recurse(
        bindings: &ObTm,
        prefix: &mut Vec<BVarSegment>,
        paths: &mut IndexMap<Name, Vec<BVarSegment>>,
    ) {
        match bindings {
            ObTm::Var(name) => {
                paths.insert(*name, prefix.clone());
            }
            ObTm::List(terms) => {
                for (i, tm) in terms.iter().enumerate() {
                    prefix.push(BVarSegment::List(i));
                    recurse(tm, prefix, paths);
                    prefix.pop();
                }
            }
            ObTm::Tensor(tm) => {
                prefix.push(BVarSegment::Tensor);
                recurse(tm, prefix, paths);
                prefix.pop();
            }
        }
    }
    let mut paths = IndexMap::new();
    recurse(bindings, &mut Vec::new(), &mut paths);
    paths
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
    Let { bound: MorTm, body: Box<PatTm> },
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

    /// Simultaneously substitutes terms for free variables in the pattern.
    ///
    /// Capture-avoiding due to the locally nameless representation.
    pub fn subst(&self, subst: &[(Name, MorTm)]) -> Self {
        match self {
            PatTm::Res(name, tm) => PatTm::res(*name, tm.subst(subst)),
            PatTm::List(patterns) => PatTm::list(patterns.iter().map(|p| p.subst(subst))),
            PatTm::Tensor(pattern) => PatTm::tensor(pattern.subst(subst)),
            PatTm::Let { bound, body } => PatTm::let_(bound.subst(subst), body.subst(subst)),
        }
    }

    /// Restricts the pattern term at free variables along a morphism term.
    ///
    /// The codomain of the morphism should equal the type of the object term.
    pub fn restrict(self, at: &ObTm, along: MorTm) -> Self {
        if let ObTm::Var(var) = at {
            self.subst(&[(*var, along)])
        } else {
            self.bind(at, along)
        }
    }

    /// Binds free variables with a `let in` expression.
    pub fn bind(self, bindings: &ObTm, bound: MorTm) -> Self {
        PatTm::let_(bound, self.close(bindings))
    }

    /// Closes over free variables by turning them into bound variables.
    pub fn close(self, bindings: &ObTm) -> Self {
        self.close_rec(&binding_paths(bindings), 0)
    }

    fn close_rec(self, paths: &IndexMap<Name, Vec<BVarSegment>>, depth: usize) -> Self {
        match self {
            PatTm::Res(name, tm) => PatTm::res(name, tm.close_rec(paths, depth)),
            PatTm::List(patterns) => {
                PatTm::list(patterns.into_iter().map(|p| p.close_rec(paths, depth)))
            }
            PatTm::Tensor(pattern) => PatTm::tensor(pattern.close_rec(paths, depth)),
            PatTm::Let { bound, body } => {
                PatTm::let_(bound.close_rec(paths, depth), body.close_rec(paths, depth + 1))
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn subst_mor() {
        // Free variables.
        let subst = vec![(name("x"), MorTm::app("f", MorTm::fvar("a")))];
        expect!["f a"].assert_eq(&MorTm::fvar("x").subst(&subst).to_string());
        expect!["y"].assert_eq(&MorTm::fvar("y").subst(&subst).to_string());

        // Lists, applications, etc.
        let tm = MorTm::app("g", [MorTm::fvar("x"), MorTm::fvar("y")]);
        expect!["g [f a, y]"].assert_eq(&tm.subst(&subst).to_string());

        // Let bindings.
        let subst = vec![(name("x"), MorTm::fvar("a")), (name("y"), MorTm::fvar("b"))];
        let tm = MorTm::let_(
            [MorTm::fvar("x"), MorTm::fvar("y")],
            [
                MorTm::bvar(0, [BVarSegment::List(0)]),
                MorTm::fvar("y"),
                MorTm::bvar(0, [BVarSegment::List(1)]),
            ],
        );
        expect!["let [x, y] in [0.0, y, 0.1]"].assert_eq(&tm.to_string());
        expect!["let [a, b] in [0.0, b, 0.1]"].assert_eq(&tm.subst(&subst).to_string());
    }

    #[test]
    fn subst_pattern() {
        let subst = vec![(name("x"), MorTm::app("f", MorTm::fvar("a")))];
        let tm = PatTm::tensor([
            PatTm::res("A", [MorTm::fvar("x")]),
            PatTm::res("B", [MorTm::fvar("y")]),
        ]);
        expect!["(A [x], B [y])"].assert_eq(&tm.to_string());
        expect!["(A [f a], B [y])"].assert_eq(&tm.subst(&subst).to_string());
    }

    #[test]
    fn bind_pattern() {
        // Binding a binary tensor product.
        let tm = PatTm::res("A", [MorTm::fvar("x"), MorTm::fvar("y")]);
        let bound = tm.bind(&ObTm::tensor([ObTm::var("x"), ObTm::var("y")]), MorTm::app("f", []));
        expect!["let f [] in A [0.0, 0.1]"].assert_eq(&bound.to_string());

        // Nested binding.
        let tm = PatTm::let_(
            MorTm::app("g", []),
            PatTm::res("A", [MorTm::fvar("x"), MorTm::bvar(0, [])]),
        );
        expect!["let g [] in A [x, 0]"].assert_eq(&tm.to_string());
        let bound = tm.bind(&ObTm::var("x"), MorTm::app("f", []));
        expect!["let f [] in let g [] in A [1, 0]"].assert_eq(&bound.to_string());
    }
}
