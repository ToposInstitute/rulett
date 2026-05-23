//! Types for rule-based models.

use derive_more::Display;
use itertools::Itertools;

use super::prelude::*;

/// A kind, or meta type.
///
/// The (meta) type of a [type](Ty) is a kind. In double-categorical logic,
/// kinds correspond to object types in the double theory.
#[derive(PartialEq, Eq, Display)]
pub enum Kind {
    /// The base or primitive kind, often denoted `*`.
    #[display("*")]
    Prim,

    /// An application of the list constructor to a kind.
    #[display("List {_0}")]
    List(Box<Kind>),

    /// A hole, representing an unknown kind.
    #[display("?")]
    Hole,
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
pub enum Ty {
    /// A primitive type, aka a sort, belonging to the signature.
    Sort(Name),

    /// A list of types, each of which should have the same kind.
    List(Vec<Ty>),

    /// An application of the tensor (`⊗: List(Prim) -> Prim`) to a type.
    Tensor(Box<Ty>),
}

impl Ty {
    /// Smart constructor for [`Sort`](Self::Sort) variant.
    pub fn sort(name: impl Into<Name>) -> Self {
        Self::Sort(name.into())
    }

    /// Smart constructor for [`List`](Self::List) variant.
    pub fn list(types: impl IntoIterator<Item = Ty>) -> Self {
        Self::List(types.into_iter().collect())
    }

    /// Smart constructor for [`Tensor`](Self::Tensor) variant.
    pub fn tensor(ty: Ty) -> Self {
        Self::Tensor(Box::new(ty))
    }

    /// Checks whether the type is of the given kind.
    pub fn check(&self, kind: &Kind) -> bool {
        self.synthesize().is_some_and(|k| k.unifies_with(kind))
    }

    /// Synthesizes a kind for the type.
    ///
    /// Returns something if the type is well-meta-typed up to holes; otherwise,
    /// returns nothing.
    pub fn synthesize(&self) -> Option<Kind> {
        match self {
            Ty::Sort(_) => Some(Kind::prim()),
            Ty::List(types) => {
                let kinds: Option<Vec<_>> = types.iter().map(|ty| ty.synthesize()).collect();
                match kinds?.into_iter().all_equal_value() {
                    Ok(kind) => Some(Kind::list(kind)),
                    Err(None) => Some(Kind::list(Kind::hole())),
                    Err(Some(_)) => None,
                }
            }
            Ty::Tensor(ty) => {
                if ty.check(&Kind::list(Kind::prim())) {
                    Some(Kind::prim())
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesize() {
        let syn = |ty: Ty| {
            ty.synthesize()
                .map(|kind| kind.to_string())
                .unwrap_or_else(|| "ERROR".to_owned())
        };

        // Sorts.
        assert_eq!(syn(Ty::sort("X")), "*");

        // Lists.
        assert_eq!(syn(Ty::list([Ty::sort("X"), Ty::sort("Y")])), "List *");
        assert_eq!(syn(Ty::list([Ty::list([Ty::sort("X")])])), "List List *");
        assert_eq!(syn(Ty::list([])), "List ?");
        assert_eq!(syn(Ty::list([Ty::sort("X"), Ty::list([])])), "ERROR");

        // Tensors.
        assert_eq!(syn(Ty::tensor(Ty::list([Ty::sort("X"), Ty::sort("Y")]))), "*");
        assert_eq!(syn(Ty::tensor(Ty::sort("X"))), "ERROR");
    }

    #[test]
    fn check() {
        // Lists.
        let kind = Kind::list(Kind::prim());
        assert!(Ty::list([Ty::sort("X")]).check(&kind));
        assert!(Ty::list([]).check(&kind));
    }
}
