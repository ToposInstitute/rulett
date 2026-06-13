//! Object terms, shared by the surface and core syntax.

use pretty::RcDoc;
use std::fmt;

use crate::{prelude::*, ty::*};

/// Object term.
///
/// More precisely, this is an object term sans type. The judgment that an
/// object term has a type is represented by [`ObTmJudgment`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
        render_doc(self.to_doc(), f)
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
    /// Pretty document for the object term.
    pub fn to_doc(&self) -> RcDoc<'static> {
        match self {
            ObTm::Var(name) => RcDoc::text(name.as_str()),
            ObTm::List(terms) => bracketed("[", "]", terms.iter().map(ObTm::to_doc)),
            ObTm::Tensor(tm) => match &**tm {
                ObTm::List(terms) => bracketed("(", ")", terms.iter().map(ObTm::to_doc)),
                _ => RcDoc::text("⊗ ").append(tm.to_doc()),
            },
        }
    }

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
        self.vars()
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
    /// In a valid object term, no variable is repeated. Returns an error
    /// containing the offending name if a variable is encountered twice.
    pub fn vars(&self) -> Result<IndexSet<Name>, Name> {
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub fn typed_vars(&self) -> IndexMap<Name, Name> {
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
}
