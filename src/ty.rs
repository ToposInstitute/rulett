//! Types for rule-based models.

use pretty::RcDoc;
use std::fmt;

use super::prelude::*;

/// A kind, or meta type.
///
/// The (meta) type of a [type](Ty) is a kind. In double-categorical logic,
/// kinds correspond to object types in the double theory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The base or primitive kind, often denoted `*`.
    Prim,

    /// An application of the list constructor to a kind.
    List(Box<Kind>),

    /// A hole, representing an unknown kind.
    Hole,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Prim => write!(f, "*"),
            Kind::List(kind) => write!(f, "List {kind}"),
            Kind::Hole => write!(f, "?"),
        }
    }
}

impl Kind {
    /// Smart constructor for [`Prim`](Self::Prim) variant.
    pub fn prim() -> Self {
        Self::Prim
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(kind: Self) -> Self {
        Self::List(Box::new(kind))
    }

    /// Smart constructor for [`Hole`](Self::Hole) variant.
    pub fn hole() -> Self {
        Self::Hole
    }

    /// Checks whether the two kinds are equal up to holes.
    pub fn unifies_with(&self, other: &Self) -> bool {
        if matches!(other, Self::Hole) {
            return true;
        }
        match self {
            Self::Hole => true,
            Self::Prim => matches!(other, Self::Prim),
            Self::List(k1) => matches!(other, Self::List(k2) if k1.unifies_with(k2)),
        }
    }
}

/// A type over a signature.
///
/// In double-categorical logic, types correspond to objects in a model of the
/// double theory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    /// A primitive type, aka a sort, belonging to the signature.
    Sort(Name),

    /// A list of types, each of which should have the same kind.
    List(Vec<Ty>),

    /// An application of the tensor (`⊗: List(Prim) -> Prim`) to a type.
    Tensor(Box<Ty>),
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        render_doc(self.to_doc(), f)
    }
}

impl FromIterator<Ty> for Ty {
    fn from_iter<T: IntoIterator<Item = Ty>>(iter: T) -> Self {
        Self::List(iter.into_iter().collect())
    }
}

impl<const N: usize> From<[Ty; N]> for Ty {
    fn from(value: [Ty; N]) -> Self {
        Self::List(value.into())
    }
}

impl Ty {
    /// Pretty document for the type.
    pub fn to_doc(&self) -> RcDoc<'static> {
        match self {
            Ty::Sort(name) => RcDoc::text(name.as_str()),
            Ty::List(types) => bracketed("[", "]", types.iter().map(Ty::to_doc)),
            Ty::Tensor(ty) => RcDoc::text("⊗ ").append(ty.to_doc()),
        }
    }

    /// Smart constructor for [`Sort`](Self::Sort) variant.
    pub fn sort(name: impl Into<Name>) -> Self {
        Self::Sort(name.into())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(types: impl IntoIterator<Item = Ty>) -> Self {
        Self::from_iter(types)
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(ty: impl Into<Ty>) -> Self {
        Self::Tensor(Box::new(ty.into()))
    }

    /// Collects all the sorts that appear in the type.
    pub fn collect_sorts(&self) -> Vec<Name> {
        fn recurse(sorts: &mut Vec<Name>, ty: &Ty) {
            match ty {
                Ty::Sort(name) => sorts.push(*name),
                Ty::List(types) => {
                    for ty in types {
                        recurse(sorts, ty);
                    }
                }
                Ty::Tensor(ty) => recurse(sorts, ty),
            }
        }
        let mut sorts = Vec::new();
        recurse(&mut sorts, self);
        sorts
    }

    /// Checks whether the type is of the given kind.
    ///
    /// Returns an error when the type is not well-kinded.
    pub fn check(&self, kind: &Kind) -> Result<bool, String> {
        self.synthesize().map(|k| k.unifies_with(kind))
    }

    /// Synthesizes a kind for the type.
    ///
    /// Returns an error when the type is not well-kinded.
    pub fn synthesize(&self) -> Result<Kind, String> {
        match self {
            Ty::Sort(_) => Ok(Kind::prim()),
            Ty::List(types) => {
                let kinds: Result<Vec<_>, _> = types.iter().map(|ty| ty.synthesize()).collect();
                match kinds?.into_iter().all_equal_value() {
                    Ok(kind) => Ok(Kind::list(kind)),
                    Err(None) => Ok(Kind::list(Kind::hole())),
                    Err(Some(_)) => Err(format!("mixed types in list: {self}")),
                }
            }
            Ty::Tensor(ty) => {
                if ty.check(&Kind::list(Kind::prim()))? {
                    Ok(Kind::prim())
                } else {
                    Err(format!("tensor should be applied to list, received: {ty}"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn collect_sorts() {
        // Sorts.
        assert_eq!(Ty::sort("X").collect_sorts(), vec![name("X")]);

        // Lists.
        let ty = Ty::list([Ty::sort("X"), Ty::sort("Y"), Ty::sort("X")]);
        assert_eq!(ty.collect_sorts(), vec![name("X"), name("Y"), name("X")]);

        // Tensors.
        let ty = Ty::tensor([Ty::sort("X"), Ty::sort("Y")]);
        assert_eq!(ty.collect_sorts(), vec![name("X"), name("Y")]);
    }

    #[test]
    fn synthesize() {
        fn syn(ty: Ty) -> String {
            match ty.synthesize() {
                Ok(kind) => kind.to_string(),
                Err(msg) => format!("ERROR: {msg}"),
            }
        }

        // Sorts.
        assert_eq!(syn(Ty::sort("X")), "*");

        // Lists.
        assert_eq!(syn(Ty::list([Ty::sort("X"), Ty::sort("Y")])), "List *");
        assert_eq!(syn(Ty::list([Ty::list([Ty::sort("X")])])), "List List *");
        assert_eq!(syn(Ty::list([])), "List ?");
        let err = expect!["ERROR: mixed types in list: [X, []]"];
        err.assert_eq(&syn(Ty::list([Ty::sort("X"), Ty::list([])])));

        // Tensors.
        assert_eq!(syn(Ty::tensor([Ty::sort("X"), Ty::sort("Y")])), "*");
        let err = expect!["ERROR: tensor should be applied to list, received: X"];
        err.assert_eq(&syn(Ty::tensor(Ty::sort("X"))));
    }

    #[test]
    fn check() {
        fn chk(ty: &Ty, kind: &Kind) -> bool {
            ty.check(kind).unwrap_or_default()
        }

        // Lists.
        let kind = Kind::list(Kind::prim());
        assert!(chk(&Ty::list([Ty::sort("X")]), &kind));
        assert!(chk(&Ty::list([]), &kind));
    }
}
