//! Terms in surface syntax.

use pretty::RcDoc;
use std::fmt;

use crate::{core, ob_tm::*, prelude::*};

/// Morphism term (sans domain term and codomain type).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MorTm {
    /// A variable.
    ///
    /// Example syntax: `x`
    Var(Name),

    /// A list of terms.
    ///
    /// Example syntax: `[x, y, z]`
    List(Vec<MorTm>),

    /// An application of the tensor product to a term.
    ///
    /// Example syntax: `⊗ [t, s]`
    Tensor(Box<MorTm>),

    /// An application of an operation in the signature to a term.
    ///
    /// Example syntax: `f t`, where `t = [x, y]`
    App(Name, Box<MorTm>),

    /// A let binding.
    ///
    /// Example syntax: `let ⊗ [x, y] = t in f [y, x]`
    Let {
        bindings: ObTm,
        bound: Box<MorTm>,
        body: Box<MorTm>,
    },
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
            MorTm::Var(name) => RcDoc::text(name.as_str()),
            MorTm::List(terms) => bracketed("[", "]", terms.iter().map(MorTm::to_doc)),
            MorTm::Tensor(tm) => match &**tm {
                MorTm::List(terms) => bracketed("(", ")", terms.iter().map(MorTm::to_doc)),
                _ => RcDoc::text("⊗ ").append(tm.to_doc()),
            },
            MorTm::App(name, tm) => {
                RcDoc::text(name.as_str()).append(RcDoc::space()).append(tm.to_doc())
            }
            MorTm::Let { bindings, bound, body } => {
                let_doc(bindings.to_doc(), bound.to_doc(), body.to_doc())
            }
        }
    }

    /// Smart constructor for [`Var`](Self::Var) variant.
    pub fn var(name: impl Into<Name>) -> Self {
        Self::Var(name.into())
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
    pub fn let_(
        bindings: impl Into<ObTm>,
        bound: impl Into<MorTm>,
        body: impl Into<MorTm>,
    ) -> Self {
        Self::Let {
            bindings: bindings.into(),
            bound: Box::new(bound.into()),
            body: Box::new(body.into()),
        }
    }

    /// Elaborates the term from surface syntax into core syntax.
    pub fn elab(&self) -> core::MorTm {
        self.elab_with(&mut Vec::new())
    }

    fn elab_with<'a>(&'a self, env: &mut Vec<&'a ObTm>) -> core::MorTm {
        match self {
            MorTm::Var(name) => elab_var(*name, env),
            MorTm::List(terms) => {
                core::MorTm::List(terms.iter().map(|tm| tm.elab_with(env)).collect())
            }
            MorTm::Tensor(tm) => core::MorTm::Tensor(Box::new(tm.elab_with(env))),
            MorTm::App(name, tm) => core::MorTm::App(*name, Box::new(tm.elab_with(env))),
            MorTm::Let { bindings, bound, body } => {
                let bound = Box::new(bound.elab_with(env));
                env.push(bindings);
                let body = Box::new(body.elab_with(env));
                env.pop();
                core::MorTm::Let { bound, body }
            }
        }
    }

    /// Simultaneously substitutes terms for free variables in the term.
    ///
    /// Warning: substitution is not capture-avoiding.
    pub fn subst(&self, subst: &mut Vec<(Name, Self)>) -> Self {
        match self {
            MorTm::Var(name) => subst
                .iter()
                .rev()
                .find_map(|(n, tm)| (n == name).then(|| tm.clone()))
                .unwrap_or_else(|| self.clone()),
            MorTm::List(terms) => MorTm::list(terms.iter().map(|t| t.subst(subst))),
            MorTm::Tensor(tm) => MorTm::tensor(tm.subst(subst)),
            MorTm::App(name, tm) => MorTm::app(*name, tm.subst(subst)),
            MorTm::Let { bindings, bound, body } => {
                let new_bound = bound.subst(subst);
                let shadowed = bindings.vars().unwrap_or_default();
                let n = shadowed.len();
                for &name in &shadowed {
                    subst.push((name, MorTm::var(name)));
                }
                let new_body = body.subst(subst);
                subst.truncate(subst.len() - n);
                MorTm::let_(bindings.clone(), new_bound, new_body)
            }
        }
    }
}

/// Pattern term in a rule-based model.
///
/// Pattern terms ("pat-terms") are used to represent both indexed objects
/// ("patterns" in Kappa) and indexed morphisms (derived rules) excluding their
/// (co)domains. In the latter case, we follow the category theorist's tradition
/// of an identifying an object with its identity morphism.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatTm {
    /// A restriction of an agent or a basic rule along a morphism.
    ///
    /// Example syntax: `A t` or `R t`, where `t = [x, y]`
    Res(Name, MorTm),

    /// A list of terms.
    ///
    /// Example syntax: `[A [x], R [y]]`
    List(Vec<PatTm>),

    /// An application of the tensor product.
    ///
    /// Example syntax: `⊗ [A [x], R [y]]`
    Tensor(Box<PatTm>),

    /// A let binding.
    ///
    /// Example syntax: `let ⊗ [x, y] = t in A [y, x]`
    ///
    /// Strictly speaking, let bindings don't belong in pattern terms---they can
    /// always be pushed into morphism terms, where they do belong---but we
    /// allow them here because (1) they're convenient in the species search
    /// algorithm and (2) they make for nicer pretty printing.
    Let {
        bindings: ObTm,
        bound: MorTm,
        body: Box<PatTm>,
    },
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
            PatTm::Let { bindings, bound, body } => {
                let_doc(bindings.to_doc(), bound.to_doc(), body.to_doc())
            }
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
    pub fn let_(
        bindings: impl Into<ObTm>,
        bound: impl Into<MorTm>,
        body: impl Into<PatTm>,
    ) -> Self {
        Self::Let {
            bindings: bindings.into(),
            bound: bound.into(),
            body: Box::new(body.into()),
        }
    }

    /// Restricts the pattern term at free variables along a morphism term.
    ///
    /// The codomain of the morphism should equal the type of the object term.
    pub fn restrict(&self, at: ObTm, along: MorTm) -> Self {
        if let ObTm::Var(var) = at {
            // In the co-unary case, substitute along a single variable.
            self.subst(&mut vec![(var, along)])
        } else {
            // Otherwise, introduce a let binding.
            Self::let_(at, along, self.clone())
        }
    }

    /// Elaborates the pattern term from surface syntax into core syntax.
    pub fn elab(&self) -> core::PatTm {
        self.elab_with(&mut Vec::new())
    }

    fn elab_with<'a>(&'a self, env: &mut Vec<&'a ObTm>) -> core::PatTm {
        match self {
            PatTm::Res(name, tm) => core::PatTm::Res(*name, tm.elab_with(env)),
            PatTm::List(patterns) => {
                core::PatTm::List(patterns.iter().map(|p| p.elab_with(env)).collect())
            }
            PatTm::Tensor(tm) => core::PatTm::Tensor(Box::new(tm.elab_with(env))),
            PatTm::Let { bindings, bound, body } => {
                let bound = bound.elab_with(env);
                env.push(bindings);
                let body = Box::new(body.elab_with(env));
                env.pop();
                core::PatTm::Let { bound, body }
            }
        }
    }

    /// Simultaneously substitutes terms for free variables in the pattern.
    ///
    /// Warning: Substitution is not capture-avoiding.
    pub fn subst(&self, subst: &mut Vec<(Name, MorTm)>) -> Self {
        match self {
            PatTm::Res(name, tm) => PatTm::res(*name, tm.subst(subst)),
            PatTm::List(patterns) => PatTm::list(patterns.iter().map(|p| p.subst(subst))),
            PatTm::Tensor(pattern) => PatTm::tensor(pattern.subst(subst)),
            PatTm::Let { bindings, bound, body } => {
                let new_bound = bound.subst(subst);
                let shadowed = bindings.vars().unwrap_or_default();
                let n = shadowed.len();
                for &name in &shadowed {
                    subst.push((name, MorTm::var(name)));
                }
                let new_body = body.subst(subst);
                subst.truncate(subst.len() - n);
                PatTm::let_(bindings.clone(), new_bound, new_body)
            }
        }
    }
}

/// Elaborates a surface variable into a free or bound core variable.
///
/// The binder stack `env` holds the binding structures of the enclosing let
/// bindings, innermost last. It is searched from innermost to outermost; the de
/// Bruijn index of a [`BVar`](core::MorTm::BVar) is the number of let bindings
/// between the use of the variable and the binding that introduces it.
fn elab_var(name: Name, env: &[&ObTm]) -> core::MorTm {
    for (index, bindings) in env.iter().rev().enumerate() {
        if let Some(mut path) = bvar_path(bindings, name) {
            // `bvar_path` builds the path innermost first; reverse it so the
            // segments are ordered outermost first, as [`BVarSegment`] expects.
            path.reverse();
            return core::MorTm::BVar(index, path);
        }
    }
    core::MorTm::FVar(name)
}

/// Computes the path from a let binding's structure to a bound variable.
///
/// Returns `None` if the variable does not occur in the binding structure. The
/// returned segments are ordered innermost first (the reverse of what
/// [`BVarSegment`] expects), since each level pushes its own segment as the
/// recursion unwinds; the caller reverses them.
///
/// [`BVarSegment`]: core::BVarSegment
fn bvar_path(bindings: &ObTm, name: Name) -> Option<Vec<core::BVarSegment>> {
    match bindings {
        ObTm::Var(n) => (*n == name).then(Vec::new),
        ObTm::List(terms) => terms.iter().enumerate().find_map(|(i, tm)| {
            bvar_path(tm, name).map(|mut path| {
                path.push(core::BVarSegment::List(i));
                path
            })
        }),
        ObTm::Tensor(tm) => bvar_path(tm, name).map(|mut path| {
            path.push(core::BVarSegment::Tensor);
            path
        }),
    }
}

/// Pretty document for `let <bindings> = <bound> in <body>`, breakable after `in`.
fn let_doc<'a>(bindings: RcDoc<'a>, bound: RcDoc<'a>, body: RcDoc<'a>) -> RcDoc<'a> {
    RcDoc::text("let ")
        .append(bindings)
        .append(RcDoc::text(" = "))
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
        // Variables.
        let mut subst = vec![(name("x"), MorTm::app("f", MorTm::var("a")))];
        expect!["f a"].assert_eq(&MorTm::var("x").subst(&mut subst).to_string());
        expect!["y"].assert_eq(&MorTm::var("y").subst(&mut subst).to_string());

        // Lists, applications, etc.
        let tm = MorTm::app("g", [MorTm::var("x"), MorTm::var("y")]);
        expect!["g [f a, y]"].assert_eq(&tm.subst(&mut subst).to_string());

        // Let bindings, with shadowing.
        let mut subst = vec![(name("x"), MorTm::var("a")), (name("y"), MorTm::var("b"))];
        let tm = MorTm::let_(
            [ObTm::var("x"), ObTm::var("z")],
            [MorTm::var("x"), MorTm::var("y")],
            [MorTm::var("x"), MorTm::var("y"), MorTm::var("z")],
        );
        expect!["let [x, z] = [x, y] in [x, y, z]"].assert_eq(&tm.to_string());
        expect!["let [x, z] = [a, b] in [x, b, z]"].assert_eq(&tm.subst(&mut subst).to_string());
        // Stack is restored after substitution.
        assert_eq!(subst.len(), 2);
    }

    #[test]
    fn subst_pattern() {
        // Basic substitution.
        let mut subst = vec![(name("x"), MorTm::app("f", MorTm::var("a")))];
        let tm =
            PatTm::tensor([PatTm::res("A", [MorTm::var("x")]), PatTm::res("B", [MorTm::var("y")])]);
        expect!["(A [x], B [y])"].assert_eq(&tm.to_string());
        expect!["(A [f a], B [y])"].assert_eq(&tm.subst(&mut subst).to_string());

        // Let bindings, with shadowing.
        let mut subst = vec![(name("x"), MorTm::var("a")), (name("y"), MorTm::var("b"))];
        let tm = PatTm::let_(
            [ObTm::var("x"), ObTm::var("z")],
            [MorTm::var("x"), MorTm::var("y")],
            PatTm::res("A", [MorTm::var("x"), MorTm::var("y"), MorTm::var("z")]),
        );
        expect!["let [x, z] = [x, y] in A [x, y, z]"].assert_eq(&tm.to_string());
        expect!["let [x, z] = [a, b] in A [x, b, z]"].assert_eq(&tm.subst(&mut subst).to_string());
        // Stack is restored after substitution.
        assert_eq!(subst.len(), 2);
    }

    #[test]
    fn elab_mor() {
        // Lone variables are free.
        expect!["x"].assert_eq(&MorTm::var("x").elab().to_string());

        // A single binding binds at index 0; other variables remain free.
        let tm =
            MorTm::let_(ObTm::var("x"), MorTm::app("f", []), [MorTm::var("x"), MorTm::var("y")]);
        expect!["let f [] in [0, y]"].assert_eq(&tm.elab().to_string());

        // A list destructure indexes into the binding by position.
        let tm = MorTm::let_(
            [ObTm::var("x"), ObTm::var("y")],
            MorTm::app("f", []),
            [MorTm::var("y"), MorTm::var("x")],
        );
        expect!["let f [] in [0.1, 0.0]"].assert_eq(&tm.elab().to_string());

        // Nested bindings: the de Bruijn index counts enclosing bindings and a
        // tensor destructure descends through an (omitted) tensor segment.
        let tm = MorTm::let_(
            ObTm::var("x"),
            MorTm::app("f", []),
            MorTm::let_(
                ObTm::tensor([ObTm::var("y"), ObTm::var("z")]),
                MorTm::app("g", []),
                [MorTm::var("x"), MorTm::var("y"), MorTm::var("z")],
            ),
        );
        expect!["let f [] in let g [] in [1, 0.0, 0.1]"].assert_eq(&tm.elab().to_string());

        // The innermost binding shadows outer ones.
        let tm = MorTm::let_(
            ObTm::var("x"),
            MorTm::app("f", []),
            MorTm::let_(ObTm::var("x"), MorTm::app("g", []), MorTm::var("x")),
        );
        expect!["let f [] in let g [] in 0"].assert_eq(&tm.elab().to_string());
    }

    #[test]
    fn elab_pattern() {
        let tm = PatTm::let_(
            ObTm::tensor([ObTm::var("x"), ObTm::var("y")]),
            MorTm::app("f", []),
            PatTm::tensor([PatTm::res("A", [MorTm::var("x")]), PatTm::res("B", [MorTm::var("y")])]),
        );
        expect!["let f [] in (A [0.0], B [0.1])"].assert_eq(&tm.elab().to_string());
    }
}
