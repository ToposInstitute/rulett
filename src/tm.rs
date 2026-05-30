//! Terms for rule-based models.

use std::fmt;

use super::{prelude::*, ty::*};

/// Object term.
///
/// More precisely, this is an object term sans type. The judgment that an
/// object term has a type is represented by [`ObTmJudgment`].
#[derive(Clone, PartialEq, Eq)]
pub enum ObTm {
    /// A variable.
    ///
    /// Example syntax: `x`
    Var(Name),

    /// A list of terms.
    ///
    /// Example syntax: `[x, y, z]`
    List(Vec<ObTm>),

    /// An application of the tensor product to a term.
    ///
    /// Example syntax: `⊗ [t, s]`
    Tensor(Box<ObTm>),
}

impl fmt::Display for ObTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObTm::Var(name) => write!(f, "{name}"),
            ObTm::List(terms) => write!(f, "[{}]", terms.iter().join(", ")),
            ObTm::Tensor(tm) => match &**tm {
                ObTm::List(terms) => write!(f, "({})", terms.iter().join(", ")),
                _ => write!(f, "⊗ {tm}"),
            },
        }
    }
}

impl FromIterator<ObTm> for ObTm {
    fn from_iter<T: IntoIterator<Item = ObTm>>(iter: T) -> Self {
        Self::List(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[ObTm; N]> for ObTm {
    fn from(value: [ObTm; N]) -> Self {
        Self::List(value.into())
    }
}

impl ObTm {
    /// Smart constructor for [`Var`](Self::Var) variant.
    pub fn var(name: impl Into<Name>) -> Self {
        Self::Var(name.into())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(terms: impl IntoIterator<Item = ObTm>) -> Self {
        Self::from_iter(terms)
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(tm: impl Into<ObTm>) -> Self {
        Self::Tensor(Box::new(tm.into()))
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
#[derive(Clone, PartialEq, Eq)]
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
        match self {
            MorTm::Var(name) => write!(f, "{name}"),
            MorTm::List(terms) => write!(f, "[{}]", terms.iter().join(", ")),
            MorTm::Tensor(tm) => match &**tm {
                MorTm::List(terms) => write!(f, "({})", terms.iter().join(", ")),
                _ => write!(f, "⊗ {tm}"),
            },
            MorTm::App(name, tm) => write!(f, "{name} {tm}"),
            MorTm::Let { bindings, bound, body } => {
                write!(f, "let {bindings} = {bound} in {body}")
            }
        }
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
                let shadowed = bindings.collect_vars().unwrap_or_default();
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
#[derive(Clone, PartialEq, Eq)]
pub enum PatTm {
    /// A restriction of an agent or a basic rule along a morphism.
    ///
    /// Example syntax: `A t` or `R t`, where `t = [x, y]`
    Res(Name, MorTm),

    /// A list of patterns.
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
        match self {
            PatTm::Res(name, tm) => write!(f, "{name} {tm}"),
            PatTm::List(patterns) => write!(f, "[{}]", patterns.iter().join(", ")),
            PatTm::Tensor(tm) => match &**tm {
                PatTm::List(terms) => write!(f, "({})", terms.iter().join(", ")),
                _ => write!(f, "⊗ {tm}"),
            },
            PatTm::Let { bindings, bound, body } => {
                write!(f, "let {bindings} = {bound} in {body}")
            }
        }
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
                let shadowed = bindings.collect_vars().unwrap_or_default();
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

/// Rule term.
///
/// A rule term represents an indexed morphism (derived rule) including its
/// domain (left-hand side) and codomain (right-hand side).
pub struct RuleTm {
    /// Term for rule itself.
    pub rule: PatTm,
    /// Term for left-hand side of rule.
    pub lhs: PatTm,
    /// Term for right-hand side of rule.
    pub rhs: PatTm,
}

impl RuleTm {
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
    pub fn restrict(&self, at: ObTm, along: MorTm) -> Self {
        Self {
            rule: self.rule.restrict(at.clone(), along.clone()),
            lhs: self.lhs.restrict(at.clone(), along.clone()),
            rhs: self.rhs.restrict(at, along),
        }
    }

    /// Simultaneously substitutes terms for free variables in the rule.
    ///
    /// Warning: Substitution is not capture-avoiding.
    pub fn subst(&self, subst: &mut Vec<(Name, MorTm)>) -> Self {
        Self {
            rule: self.rule.subst(subst),
            lhs: self.lhs.subst(subst),
            rhs: self.rhs.subst(subst),
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
        let tm = ObTm::tensor([ObTm::var("x"), ObTm::var("y")]);
        let ty = Ty::tensor([Ty::sort("X"), Ty::sort("Y")]);
        assert!(tm.check(&ty).is_ok());
        let err = expect!["tensor term should have tensor type: (x, y) vs X"];
        err.assert_eq(&tm.check(&Ty::sort("X")).unwrap_err());
    }

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
}
