use derive_more::Display;
use itertools::join;

use super::{prelude::*, ty::*};

/// Object term.
#[derive(PartialEq, Eq, Display)]
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
        self.check_types(ty)?;
        self.collect_vars().map_err(|name| format!("variable {name} used twice"))?;
        Ok(())
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
        fn recurse(tm: &ObTm, vars: &mut IndexSet<Name>) -> Result<(), Name> {
            match tm {
                ObTm::Var(name) => {
                    if !vars.insert(*name) {
                        return Err(*name);
                    }
                }
                ObTm::List(terms) => {
                    for tm in terms {
                        recurse(tm, vars)?;
                    }
                }
                ObTm::Tensor(tm) => recurse(tm, vars)?,
            }
            Ok(())
        }
        let mut vars = IndexSet::new();
        recurse(self, &mut vars)?;
        Ok(vars)
    }
}

/// Morphism term.
///
/// A term in context, modulo alpha-equivalence, is a morphism in the theory.
pub enum MorTm {
    /// A variable.
    ///
    /// Example syntax: `x`
    Var(Name),

    /// A list of terms.
    ///
    /// Example syntax: `[x, y, z]`
    List(Vec<MorTm>),

    /// An application of the tensor to a term.
    ///
    /// Example syntax: `⊗ [t, s]`
    Tensor(Box<MorTm>),

    /// An application of an operation in the signature to a term.
    ///
    /// Example syntax: `f t`, `f [x, y]`
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
}
