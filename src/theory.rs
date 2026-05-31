//! Theories for rule-based models.
//!
//! A theory is not an algebraic theory but a *linear* theory. We take this to
//! mean a representable symmetric multicategory. Representable symmetric
//! multicategories are equivalent (in the 2-categorical sense) to symmetric
//! monoidal categories, but starting with the multicategory structure leads to
//! a more ergonomic type-theoretic notation.

use pretty::RcDoc;
use std::fmt;

use super::{prelude::*, ty::*};

/// Declaration in the definition of a signature.
pub enum SignatureDecl {
    /// Declaration of a sort.
    Sort(Name),

    /// Declaration of an operation.
    ///
    /// Any sorts mentioned in the (co)domain must have been previously
    /// declared.
    Operation(Name, Ty, Ty),
}

impl SignatureDecl {
    /// Smart constructor for [`Sort`](Self::Sort) variant.
    pub fn sort(name: impl Into<Name>) -> Self {
        Self::Sort(name.into())
    }

    /// Smart constructor for [`Operation`](Self::Operation) variant.
    pub fn operation(name: impl Into<Name>, dom: impl Into<Ty>, cod: impl Into<Ty>) -> Self {
        Self::Operation(name.into(), dom.into(), cod.into())
    }
}

/// Signature for a rule-based model.
///
/// A signature freely generates a theory. It consists of sets of sorts and
/// operations.
#[derive(Clone, Default)]
pub struct Signature {
    sorts: IndexSet<Name>,
    operations: IndexMap<Name, (Ty, Ty)>,
}

impl Signature {
    /// Constructs an empty signature.
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses a signature from a list of declarations.
    ///
    /// If a signature is returned, it is guaranteed to be valid; otherwise, the
    /// first error encountered is reported.
    pub fn parse(decls: impl IntoIterator<Item = SignatureDecl>) -> Result<Self, String> {
        let mut sig = Self::new();
        for decl in decls {
            sig.declare(decl)?;
        }
        Ok(sig)
    }

    /// Adds a declaration to the signature.
    pub fn declare(&mut self, decl: SignatureDecl) -> Result<(), String> {
        match decl {
            SignatureDecl::Sort(name) => {
                self.add_sort(name).map_err(|err| format!("cannot declare sort {name}: {err}"))
            }
            SignatureDecl::Operation(name, dom, cod) => self
                .add_operation(name, dom, cod)
                .map_err(|err| format!("cannot declare operation {name}: {err}")),
        }
    }

    /// Adds a sort with the given name to the signature.
    pub fn add_sort(&mut self, name: Name) -> Result<(), String> {
        if !self.sorts.insert(name) {
            return Err(format!("{name} already defined"));
        }
        Ok(())
    }

    /// Adds an operation with given name to the signature.
    pub fn add_operation(&mut self, name: Name, dom: Ty, cod: Ty) -> Result<(), String> {
        if !self
            .check_ty(&dom, &Kind::list(Kind::prim()))
            .map_err(|err| format!("invalid domain: {err}"))?
        {
            return Err(format!("domain should be a list of sorts, received: {dom}"));
        }
        if !self
            .check_ty(&cod, &Kind::prim())
            .map_err(|err| format!("invalid codomain: {err}"))?
        {
            return Err(format!("codomain should be a single sort, received: {cod}"));
        }
        if self.operations.insert(name, (dom, cod)).is_some() {
            return Err(format!("{name} already defined"));
        }
        Ok(())
    }

    /// Iterates over the sorts in the signature.
    pub fn sorts(&self) -> impl Iterator<Item = Name> {
        self.sorts.iter().copied()
    }

    /// Iterates over the operations in the signature.
    pub fn operations(&self) -> impl Iterator<Item = (Name, &Ty, &Ty)> {
        self.operations.iter().map(|(name, (dom, cod))| (*name, dom, cod))
    }

    /// Gets the interface of an operation, if it exists.
    pub fn interface(&self, name: &Name) -> Option<(&Ty, &Ty)> {
        self.operations.get(name).map(|(dom, cod)| (dom, cod))
    }

    /// Checks a type against the kind and signature.
    ///
    /// Returns an error when the type is not well-kinded or has sorts not
    /// contained in the signature.
    pub fn check_ty(&self, ty: &Ty, kind: &Kind) -> Result<bool, String> {
        self.has_sorts_in(ty)
            .map_err(|name| format!("no such sort {name}"))
            .and_then(|_| ty.check(kind))
    }

    fn has_sorts_in(&self, ty: &Ty) -> Result<(), Name> {
        match ty {
            Ty::Sort(name) => {
                if !self.sorts.contains(name) {
                    return Err(*name);
                }
            }
            Ty::List(types) => {
                for ty in types {
                    self.has_sorts_in(ty)?;
                }
            }
            Ty::Tensor(ty) => {
                self.has_sorts_in(ty)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "#/ sorts:")?;
        for sort in self.sorts() {
            writeln!(f, "{sort}")?;
        }
        writeln!(f, "#/ operations:")?;
        for (op, dom, cod) in self.operations() {
            let doc = mor_doc(RcDoc::text(op.as_str()), dom.to_doc(), cod.to_doc());
            render_doc(doc, f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Signature for a toy model (variant 1).
#[cfg(test)]
pub(crate) fn toy_signature_v1() -> Signature {
    Signature::parse([
        SignatureDecl::sort("Res"),
        SignatureDecl::operation("unphos", [], Ty::sort("Res")),
        SignatureDecl::operation("phos", [], Ty::sort("Res")),
        SignatureDecl::sort("Site"),
        SignatureDecl::operation("empty", [], Ty::sort("Site")),
        SignatureDecl::operation("bond", [], Ty::tensor([Ty::sort("Site"), Ty::sort("Site")])),
    ])
    .unwrap()
}

/// Signature for a toy model (variant 2).
#[cfg(test)]
pub(crate) fn toy_signature_v2() -> Signature {
    Signature::parse([
        SignatureDecl::sort("Res"),
        SignatureDecl::operation("unphos", [], Ty::sort("Res")),
        SignatureDecl::operation("phos", [], Ty::sort("Res")),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        SignatureDecl::operation("emptyA", [], Ty::sort("SiteA")),
        SignatureDecl::operation("emptyB", [], Ty::sort("SiteB")),
        SignatureDecl::operation("bond", [], Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")])),
    ])
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn parse() {
        let expected = expect![[r#"
            #/ sorts:
            Res
            Site
            #/ operations:
            unphos : [] → Res
            phos : [] → Res
            empty : [] → Site
            bond : [] → ⊗ [Site, Site]
        "#]];
        expected.assert_eq(&toy_signature_v1().to_string());

        let expected = expect![[r#"
            #/ sorts:
            Res
            SiteA
            SiteB
            #/ operations:
            unphos : [] → Res
            phos : [] → Res
            emptyA : [] → SiteA
            emptyB : [] → SiteB
            bond : [] → ⊗ [SiteA, SiteB]
        "#]];
        expected.assert_eq(&toy_signature_v2().to_string());
    }
}
