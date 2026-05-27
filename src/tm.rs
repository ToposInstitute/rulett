use derive_more::Display;
use itertools::join;

use super::{prelude::*, ty::*};

/// Object term.
///
/// More precisely, this is an object term sans type. The judgment that an
/// object term has a type is represented by [`ObTmJudgment`].
#[derive(Clone, PartialEq, Eq, Display)]
pub enum ObTm {
    /// A variable.
    ///
    /// Example syntax: `x`
    #[display("{_0}")]
    Var(Name),

    /// A list of terms.
    ///
    /// Example syntax: `[x, y, z]`
    #[display("[{}]", join(_0, ", "))]
    List(Vec<ObTm>),

    /// An application of the tensor to a term.
    ///
    /// Example syntax: `⊗ [t, s]`
    #[display("⊗ {_0}")]
    Tensor(Box<ObTm>),
}

impl ObTm {
    /// Smart constructor for [`Var`](Self::Var) variant.
    pub fn var(name: impl Into<Name>) -> Self {
        Self::Var(name.into())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(terms: impl IntoIterator<Item = ObTm>) -> Self {
        Self::List(terms.into_iter().collect())
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(tm: ObTm) -> Self {
        Self::Tensor(Box::new(tm))
    }

    /// Checks whether the term has the given type.
    ///
    /// Returns an error when the term is ill-formed or ill-typed.
    pub fn check(&self, ty: &Ty) -> Result<(), String> {
        self.collect_vars()
            .map_err(|name| format!("variable {name} used twice"))
            .and_then(|_| self.check_types(ty))
    }

    fn check_types(&self, ty: &Ty) -> Result<(), String> {
        match self {
            ObTm::Var(name) => match ty {
                Ty::Sort(_) => Ok(()),
                _ => Err(format!("variable {name} should have primitive type, received: {ty}")),
            },
            ObTm::List(terms) => match ty {
                Ty::List(types) => {
                    if terms.len() != types.len() {
                        return Err(format!(
                            "list term and type have different lengths: {self} vs {ty}"
                        ));
                    }
                    for (tm, ty) in terms.iter().zip(types) {
                        tm.check(ty)?;
                    }
                    Ok(())
                }
                _ => Err(format!("list term should have list type: {self} vs {ty}")),
            },
            ObTm::Tensor(tm) => match ty {
                Ty::Tensor(ty) => tm.check(ty),
                _ => Err(format!("tensor term should have tensor type: {self} vs {ty}")),
            },
        }
    }

    /// Collects all the variables that appear in the term.
    ///
    /// Returns an error containing the offending name if a variable is
    /// encountered twice.
    pub fn collect_vars(&self) -> Result<IndexSet<Name>, Name> {
        fn recurse(vars: &mut IndexSet<Name>, tm: &ObTm) -> Result<(), Name> {
            match tm {
                ObTm::Var(name) => {
                    if !vars.insert(*name) {
                        return Err(*name);
                    }
                }
                ObTm::List(terms) => {
                    for tm in terms {
                        recurse(vars, tm)?;
                    }
                }
                ObTm::Tensor(tm) => recurse(vars, tm)?,
            }
            Ok(())
        }
        let mut vars = IndexSet::new();
        recurse(&mut vars, self)?;
        Ok(vars)
    }
}

/// Judgment that an object term has a type.
#[derive(Clone, PartialEq, Eq)]
pub struct ObTmJudgment {
    /// The underlying term.
    pub tm: ObTm,
    /// The underlying type.
    pub ty: Ty,
}

impl ObTmJudgment {
    /// Judges that the given term has the given type, or returns an error.
    ///
    /// While a raw constructor is allowed for efficiency in
    /// correct-by-construction algorithms, this is the preferred way to
    /// construct a judgment, as it guarantees that the judgment is valid.
    pub fn judge(tm: ObTm, ty: Ty) -> Result<Self, String> {
        tm.check(&ty)?;
        Ok(Self { tm, ty })
    }

    /// Collects variable-sort pairs from the judgment.
    ///
    /// Never panics but the result is undefined if the judgment is invalid.
    pub fn collect_typed_vars(&self) -> IndexMap<Name, Name> {
        fn recurse(vars: &mut IndexMap<Name, Name>, tm: &ObTm, ty: &Ty) {
            match tm {
                ObTm::Var(name) => {
                    if let Ty::Sort(sort) = ty {
                        vars.insert(*name, *sort);
                    }
                }
                ObTm::List(terms) => {
                    if let Ty::List(types) = ty {
                        for (tm, ty) in terms.iter().zip(types) {
                            recurse(vars, tm, ty);
                        }
                    }
                }
                ObTm::Tensor(tm) => {
                    if let Ty::Tensor(ty) = ty {
                        recurse(vars, tm, ty);
                    }
                }
            }
        }
        let mut vars = IndexMap::new();
        recurse(&mut vars, &self.tm, &self.ty);
        vars
    }
}

/// Morphism term (sans domain term and codomain type).
#[derive(Clone, PartialEq, Eq, Display)]
pub enum MorTm {
    /// A variable.
    ///
    /// Example syntax: `x`
    #[display("{_0}")]
    Var(Name),

    /// A list of terms.
    ///
    /// Example syntax: `[x, y, z]`
    #[display("[{}]", join(_0, ", "))]
    List(Vec<MorTm>),

    /// An application of the tensor product to a term.
    ///
    /// Example syntax: `⊗ [t, s]`
    #[display("⊗ {_0}")]
    Tensor(Box<MorTm>),

    /// An application of an operation in the signature to a term.
    ///
    /// Example syntax: `f t`, where `t = [x, y]`
    #[display("{_0} {_1}")]
    App(Name, Box<MorTm>),

    /// A let binding.
    ///
    /// Example syntax: `let ⊗ [x, y] = t in f [y, x]`
    #[display("let {bindings} = {bound} in {body}")]
    Let {
        bindings: ObTm,
        bound: Box<MorTm>,
        body: Box<MorTm>,
    },
}

impl MorTm {
    /// Smart constructor for [`Var`](Self::Var) variant.
    pub fn var(name: impl Into<Name>) -> Self {
        Self::Var(name.into())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(terms: impl IntoIterator<Item = MorTm>) -> Self {
        Self::List(terms.into_iter().collect())
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(tm: MorTm) -> Self {
        Self::Tensor(Box::new(tm))
    }

    /// Smart constructor for [`App`](Self::App) variant.
    pub fn app(name: impl Into<Name>, tm: MorTm) -> Self {
        Self::App(name.into(), Box::new(tm))
    }

    /// Smart constructor for [`Let`](Self::Let) variant.
    pub fn let_(bindings: ObTm, bound: MorTm, body: MorTm) -> Self {
        Self::Let {
            bindings,
            bound: Box::new(bound),
            body: Box::new(body),
        }
    }

    /// Simultaneously substitutes terms for free variables in the term.
    ///
    /// Warning: substitution is not capture-avoiding.
    pub fn subst(&self, subst: &mut Vec<(Name, MorTm)>) -> MorTm {
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
                let shadowed = bindings.collect_vars().unwrap_or_default();
                let n = shadowed.len();
                for name in &shadowed {
                    subst.push((*name, MorTm::var(*name)));
                }
                let new_body = body.subst(subst);
                subst.truncate(subst.len() - n);
                MorTm::let_(bindings.clone(), new_bound, new_body)
            }
        }
    }
}

/// Pattern term in a rule-based model.
#[derive(Clone, PartialEq, Eq, Display)]
pub enum PatternTm {
    /// A restriction of an agent along a morphism.
    ///
    /// Example syntax: `A t`, where `t = [x, y]`
    #[display("{_0} {_1}")]
    Restrict(Name, MorTm),

    /// A list of patterns.
    ///
    /// Example syntax: `[A [x], B [y]]`
    #[display("[{}]", join(_0, ", "))]
    List(Vec<PatternTm>),

    /// An application of the tensor product to a pattern.
    ///
    /// Example syntax: `⊗ [A [x], B [y]]`
    #[display("⊗ {_0}")]
    Tensor(Box<PatternTm>),

    /// A let binding.
    ///
    /// Example syntax: `let ⊗ [x, y] = t in A [y, x]`
    ///
    /// Strictly speaking, let bindings don't belong in pattern terms---they can
    /// always be pushed into morphism terms, where they do belong---but we
    /// allow them here because (1) they're convenient in the species search
    /// algorithm and (2) they make for nicer pretty printing.
    #[display("let {bindings} = {bound} in {body}")]
    Let {
        bindings: ObTm,
        bound: MorTm,
        body: Box<PatternTm>,
    },
}

impl PatternTm {
    /// Smart constructor for [`Restriction`](Self::Restriction) variant.
    pub fn restrict(name: impl Into<Name>, tm: MorTm) -> Self {
        Self::Restrict(name.into(), tm)
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(patterns: impl IntoIterator<Item = PatternTm>) -> Self {
        Self::List(patterns.into_iter().collect())
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(pattern: PatternTm) -> Self {
        Self::Tensor(Box::new(pattern))
    }

    /// Smart constructor for [`Let`](Self::Let) variant.
    pub fn let_(bindings: ObTm, bound: MorTm, body: PatternTm) -> Self {
        Self::Let { bindings, bound, body: Box::new(body) }
    }

    /// Simultaneously substitutes terms for free variables in the pattern.
    ///
    /// Warning: Substitution is not capture-avoiding.
    pub fn subst(&self, subst: &mut Vec<(Name, MorTm)>) -> PatternTm {
        match self {
            PatternTm::Restrict(name, tm) => PatternTm::restrict(*name, tm.subst(subst)),
            PatternTm::List(patterns) => PatternTm::list(patterns.iter().map(|p| p.subst(subst))),
            PatternTm::Tensor(pattern) => PatternTm::tensor(pattern.subst(subst)),
            PatternTm::Let { bindings, bound, body } => {
                let new_bound = bound.subst(subst);
                let shadowed = bindings.collect_vars().unwrap_or_default();
                let n = shadowed.len();
                for name in &shadowed {
                    subst.push((*name, MorTm::var(*name)));
                }
                let new_body = body.subst(subst);
                subst.truncate(subst.len() - n);
                PatternTm::let_(bindings.clone(), new_bound, new_body)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn check_ob() {
        // Variables.
        assert!(ObTm::var("x").check(&Ty::sort("X")).is_ok());
        let err = expect!["variable x should have primitive type, received: [X]"];
        err.assert_eq(&ObTm::var("x").check(&Ty::list([Ty::sort("X")])).unwrap_err());

        // Lists.
        let tm = ObTm::list([ObTm::var("x"), ObTm::var("y")]);
        assert!(tm.check(&Ty::list([Ty::sort("X"), Ty::sort("Y")])).is_ok());
        assert!(ObTm::list([]).check(&Ty::list([])).is_ok());
        let err = expect!["list term and type have different lengths: [x, y] vs [X]"];
        err.assert_eq(&tm.check(&Ty::list([Ty::sort("X")])).unwrap_err());
        let err = expect!["list term should have list type: [x, y] vs X"];
        err.assert_eq(&tm.check(&Ty::sort("X")).unwrap_err());
        let tm = ObTm::list([ObTm::var("x"), ObTm::var("x")]);
        let err = expect!["variable x used twice"];
        err.assert_eq(&tm.check(&Ty::list([Ty::sort("X"), Ty::sort("X")])).unwrap_err());

        // Tensors.
        let tm = ObTm::tensor(ObTm::list([ObTm::var("x"), ObTm::var("y")]));
        let ty = Ty::tensor(Ty::list([Ty::sort("X"), Ty::sort("Y")]));
        assert!(tm.check(&ty).is_ok());
        let err = expect!["tensor term should have tensor type: ⊗ [x, y] vs X"];
        err.assert_eq(&tm.check(&Ty::sort("X")).unwrap_err());
    }

    #[test]
    fn subst_mor() {
        // Variables.
        let mut subst = vec![(name("x"), MorTm::app("f", MorTm::var("a")))];
        expect!["f a"].assert_eq(&MorTm::var("x").subst(&mut subst).to_string());
        expect!["y"].assert_eq(&MorTm::var("y").subst(&mut subst).to_string());

        // Lists, applications, etc.
        let tm = MorTm::app("g", MorTm::list([MorTm::var("x"), MorTm::var("y")]));
        expect!["g [f a, y]"].assert_eq(&tm.subst(&mut subst).to_string());

        // Let bindings, with shadowing.
        let mut subst = vec![(name("x"), MorTm::var("a")), (name("y"), MorTm::var("b"))];
        let tm = MorTm::let_(
            ObTm::list([ObTm::var("x"), ObTm::var("z")]),
            MorTm::list([MorTm::var("x"), MorTm::var("y")]),
            MorTm::list([MorTm::var("x"), MorTm::var("y"), MorTm::var("z")]),
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
        let tm = PatternTm::tensor(PatternTm::list([
            PatternTm::restrict("A", MorTm::list([MorTm::var("x")])),
            PatternTm::restrict("B", MorTm::list([MorTm::var("y")])),
        ]));
        expect!["⊗ [A [x], B [y]]"].assert_eq(&tm.to_string());
        expect!["⊗ [A [f a], B [y]]"].assert_eq(&tm.subst(&mut subst).to_string());

        // Let bindings, with shadowing.
        let mut subst = vec![(name("x"), MorTm::var("a")), (name("y"), MorTm::var("b"))];
        let tm = PatternTm::let_(
            ObTm::list([ObTm::var("x"), ObTm::var("z")]),
            MorTm::list([MorTm::var("x"), MorTm::var("y")]),
            PatternTm::restrict(
                "A",
                MorTm::list([MorTm::var("x"), MorTm::var("y"), MorTm::var("z")]),
            ),
        );
        expect!["let [x, z] = [x, y] in A [x, y, z]"].assert_eq(&tm.to_string());
        expect!["let [x, z] = [a, b] in A [x, b, z]"].assert_eq(&tm.subst(&mut subst).to_string());
        // Stack is restored after substitution.
        assert_eq!(subst.len(), 2);
    }
}
