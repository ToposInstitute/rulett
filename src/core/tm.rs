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

    /// Returns whether a bound variable at the given depth is used.
    fn uses_bvar(&self, depth: usize) -> bool {
        match self {
            MorTm::FVar(_) => false,
            MorTm::BVar(index, _) => *index == depth,
            MorTm::List(terms) => terms.iter().any(|t| t.uses_bvar(depth)),
            MorTm::Tensor(tm) => tm.uses_bvar(depth),
            MorTm::App(_, tm) => tm.uses_bvar(depth),
            MorTm::Let { bound, body } => bound.uses_bvar(depth) || body.uses_bvar(depth + 1),
        }
    }

    /// Decrements the index of bound variables referring past the given depth.
    fn shift_down(&mut self, depth: usize) {
        match self {
            MorTm::FVar(_) => {}
            MorTm::BVar(index, _) => {
                if *index > depth {
                    *index -= 1;
                }
            }
            MorTm::List(terms) => terms.iter_mut().for_each(|t| t.shift_down(depth)),
            MorTm::Tensor(tm) => tm.shift_down(depth),
            MorTm::App(_, tm) => tm.shift_down(depth),
            MorTm::Let { bound, body } => {
                bound.shift_down(depth);
                body.shift_down(depth + 1);
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

    /// Collect terms from a tensor product at the top level.
    pub fn collect_tensor(self) -> Vec<Self> {
        match self {
            PatTm::Tensor(tm) => match *tm {
                PatTm::List(terms) => terms,
                _ => vec![PatTm::Tensor(tm)],
            },
            _ => vec![self],
        }
    }

    /// Factorizes the pattern by pushing let bindings into (tensors of) lists.
    ///
    /// A let binding is pushed into a list when exactly one item uses the bound
    /// variable. (Since the type theory is linear, in a valid term, this item
    /// will then itself use the bound variable exactly once.)
    pub fn factorize(self) -> Self {
        match self {
            PatTm::Res(..) => self,
            PatTm::List(patterns) => PatTm::list(patterns.into_iter().map(Self::factorize)),
            PatTm::Tensor(tm) => PatTm::tensor(tm.factorize()),
            PatTm::Let { bound, body } => Self::factorize_let(bound, body.factorize()),
        }
    }

    fn factorize_let(bound: MorTm, body: PatTm) -> PatTm {
        match body {
            PatTm::Tensor(inner) if matches!(&*inner, PatTm::List(_)) => {
                PatTm::tensor(Self::factorize_let(bound, *inner))
            }
            PatTm::List(mut terms) => {
                // Get the unique term, if any, using the let binding at this level.
                let unique_user = terms
                    .iter()
                    .enumerate()
                    .filter_map(|(i, tm)| tm.uses_bvar(0).then_some(i))
                    .exactly_one();

                if let Ok(i) = unique_user {
                    // The other terms lose this enclosing binder, so shift their
                    // bound variables down to compensate.
                    for (j, tm) in terms.iter_mut().enumerate() {
                        if j != i {
                            tm.shift_down(0);
                        }
                    }
                    let tm = std::mem::replace(&mut terms[i], PatTm::List(vec![]));
                    terms[i] = Self::factorize_let(bound, tm);
                    PatTm::list(terms)
                } else {
                    PatTm::let_(bound, PatTm::list(terms))
                }
            }
            other => PatTm::let_(bound, other),
        }
    }

    /// Returns whether a bound variable at the given depth is used.
    fn uses_bvar(&self, depth: usize) -> bool {
        match self {
            PatTm::Res(_, tm) => tm.uses_bvar(depth),
            PatTm::List(patterns) => patterns.iter().any(|p| p.uses_bvar(depth)),
            PatTm::Tensor(pattern) => pattern.uses_bvar(depth),
            PatTm::Let { bound, body } => bound.uses_bvar(depth) || body.uses_bvar(depth + 1),
        }
    }

    /// Decrements the index of bound variables referring past the given depth.
    fn shift_down(&mut self, depth: usize) {
        match self {
            PatTm::Res(_, tm) => tm.shift_down(depth),
            PatTm::List(patterns) => patterns.iter_mut().for_each(|p| p.shift_down(depth)),
            PatTm::Tensor(pattern) => pattern.shift_down(depth),
            PatTm::Let { bound, body } => {
                bound.shift_down(depth);
                body.shift_down(depth + 1);
            }
        }
    }
}

/// Rule term.
///
/// A rule term represents an indexed morphism (derived rule) including its
/// domain (left-hand side) and codomain (right-hand side).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleTm {
    /// Term for rule itself.
    pub rule: PatTm,
    /// Term for left-hand side of rule.
    pub lhs: PatTm,
    /// Term for right-hand side of rule.
    pub rhs: PatTm,
}

impl fmt::Display for RuleTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        render_doc(self.to_doc(), f)
    }
}

impl RuleTm {
    /// Pretty document for the rule term.
    pub fn to_doc(&self) -> RcDoc<'static> {
        mor_doc(self.rule.to_doc(), self.lhs.to_doc(), self.rhs.to_doc())
    }

    /// Constructs a list of rule terms.
    pub fn list(rules: Vec<RuleTm>) -> Self {
        let n = rules.len();
        let (mut rule, mut lhs, mut rhs) =
            (Vec::with_capacity(n), Vec::with_capacity(n), Vec::with_capacity(n));
        for r in rules {
            rule.push(r.rule);
            lhs.push(r.lhs);
            rhs.push(r.rhs);
        }
        Self {
            rule: PatTm::list(rule),
            lhs: PatTm::list(lhs),
            rhs: PatTm::list(rhs),
        }
    }

    /// Constructs an application of the tensor product to a rule term.
    pub fn tensor(rule: RuleTm) -> Self {
        Self {
            rule: PatTm::tensor(rule.rule),
            lhs: PatTm::tensor(rule.lhs),
            rhs: PatTm::tensor(rule.rhs),
        }
    }

    /// Restricts the rule term at free variables along a morphism term.
    pub fn restrict(self, at: &ObTm, along: MorTm) -> Self {
        Self {
            rule: self.rule.restrict(at, along.clone()),
            lhs: self.lhs.restrict(at, along.clone()),
            rhs: self.rhs.restrict(at, along),
        }
    }

    /// Simultaneously substitutes terms for free variables in the rule.
    ///
    /// Capture-avoiding due to the locally nameless representation.
    pub fn subst(&self, subst: &[(Name, MorTm)]) -> Self {
        Self {
            rule: self.rule.subst(subst),
            lhs: self.lhs.subst(subst),
            rhs: self.rhs.subst(subst),
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

    #[test]
    fn collect_pattern() {
        let a = PatTm::res("A", [MorTm::app("f", [])]);
        let b = PatTm::res("B", [MorTm::app("g", [])]);
        let ab = [a.clone(), b.clone()];
        assert_eq!(PatTm::tensor(ab.clone()).collect_tensor(), ab);
        assert_eq!(a.clone().collect_tensor(), vec![a]);
    }

    #[test]
    fn factorize_pattern() {
        let tm = PatTm::tensor([
            PatTm::res("A", [MorTm::fvar("x"), MorTm::fvar("y")]),
            PatTm::res("B", []),
        ])
        .bind(&ObTm::tensor([ObTm::var("x"), ObTm::var("y")]), MorTm::app("f", []));
        expect!["(let f [] in A [0.0, 0.1], B [])"].assert_eq(&tm.factorize().to_string());

        // Wraps a body in `let x = f [] in let y = g [] in ...`.
        fn bind(body: PatTm) -> PatTm {
            body.bind(&ObTm::var("y"), MorTm::app("g", []))
                .bind(&ObTm::var("x"), MorTm::app("f", []))
        }

        let tm = bind(PatTm::tensor([
            PatTm::res("A", [MorTm::fvar("x"), MorTm::fvar("y")]),
            PatTm::res("B", []),
        ]));
        expect!["(let f [] in let g [] in A [1, 0], B [])"].assert_eq(&tm.factorize().to_string());

        let tm = bind(PatTm::tensor([
            PatTm::res("A", [MorTm::fvar("x")]),
            PatTm::res("B", [MorTm::fvar("y")]),
        ]));
        expect!["(let f [] in A [0], let g [] in B [0])"].assert_eq(&tm.factorize().to_string());

        let tm = bind(PatTm::tensor([
            PatTm::res("A", [MorTm::fvar("y")]),
            PatTm::res("B", [MorTm::fvar("x")]),
        ]));
        expect!["(let g [] in A [0], let f [] in B [0])"].assert_eq(&tm.factorize().to_string());
    }
}
