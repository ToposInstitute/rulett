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
            render_doc(mor_doc(RcDoc::text(op.as_str()), dom.to_doc(), cod.to_doc()), f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
/// Function to merge two signatures
fn merge_signatures(sigs: &[Signature]) -> Signature {
    let mut merged = Signature::new();
    for sig in sigs {
        for sort in sig.sorts() {
            if !merged.sorts().any(|s| s == sort) {
                merged.add_sort(sort).unwrap();
            }
        }
        for (op, dom, cod) in sig.operations() {
            if merged.interface(&op).is_none() {
                merged.add_operation(op, dom.clone(), cod.clone()).unwrap();
            }
        }
    }
    merged
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

/// Signature for a toy model (single agent).
#[cfg(test)]
pub(crate) fn toy_signature_single_agent() -> Signature {
    Signature::parse([
        SignatureDecl::sort("ReMonomer"),
        SignatureDecl::sort("ReA"),
        SignatureDecl::sort("ReB"),
        SignatureDecl::sort("ReB1"),
        SignatureDecl::sort("ReB2"),
        SignatureDecl::sort("ReK"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        SignatureDecl::sort("Res"),
        SignatureDecl::operation("iota_A", [Ty::sort("ReA")], Ty::sort("ReMonomer")),
        SignatureDecl::operation("iota_B", [Ty::sort("ReB")], Ty::sort("ReMonomer")),
        SignatureDecl::operation("iota_B1", [Ty::sort("ReB1")], Ty::sort("ReB")),
        SignatureDecl::operation("iota_B2", [Ty::sort("ReB2")], Ty::sort("ReB")),
        SignatureDecl::operation("iota_K", [Ty::sort("ReK")], Ty::sort("ReMonomer")),
        SignatureDecl::operation("iota_SiteA", [Ty::sort("SiteA")], Ty::sort("ReMonomer")),
        SignatureDecl::operation("iota_SiteB", [Ty::sort("SiteB")], Ty::sort("ReMonomer")),
        SignatureDecl::operation("iota_Res", [Ty::sort("Res")], Ty::sort("ReMonomer")),
        SignatureDecl::operation("phos", [], Ty::sort("Res")),
        SignatureDecl::operation("unphos", [], Ty::sort("Res")),
        SignatureDecl::operation("ground_A", [], Ty::sort("ReA")),
        SignatureDecl::operation("ground_K", [], Ty::sort("ReK")),
        SignatureDecl::operation("emptyA", [], Ty::sort("SiteA")),
        SignatureDecl::operation("emptyB", [], Ty::sort("SiteB")),
        SignatureDecl::operation("bond", [], Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")])),
    ])
    .unwrap()
}

#[cfg(test)]
fn toy_signature_one_b() -> Signature {
    Signature::parse([
        SignatureDecl::sort("ReB"),
        SignatureDecl::operation("ground_B", [], Ty::sort("ReB")),
    ])
    .unwrap()
}

#[cfg(test)]
fn toy_signature_two_b() -> Signature {
    Signature::parse([
        SignatureDecl::sort("ReB1"),
        SignatureDecl::sort("ReB2"),
        SignatureDecl::operation("ground_B1", [], Ty::sort("ReB1")),
        SignatureDecl::operation("ground_B2", [], Ty::sort("ReB2")),
    ])
    .unwrap()
}

/// Signature for a toy model (species granularity).
#[cfg(test)]
pub(crate) fn toy_signature_species_granularity_1() -> Signature {
    let sig1 = toy_signature_single_agent();
    let sig2 = toy_signature_one_b();
    merge_signatures(&[sig1, sig2])
}

/// Signature for a toy model (species granularity).
#[cfg(test)]
pub(crate) fn toy_signature_species_granularity_2() -> Signature {
    let sig1 = toy_signature_single_agent();
    let sig2 = toy_signature_two_b();
    merge_signatures(&[sig1, sig2])
}

/// Signature for a toy model (emergent agent (dimerization of A and B creates C-binding ability)).
#[cfg(test)]
pub(crate) fn toy_signature_emergent_agent() -> Signature {
    Signature::parse([
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        SignatureDecl::sort("SiteC"),
        SignatureDecl::sort("SiteAB"),
        SignatureDecl::operation("e_A", [], Ty::sort("SiteA")),
        SignatureDecl::operation("e_B", [], Ty::sort("SiteB")),
        SignatureDecl::operation("e_C", [], Ty::sort("SiteC")),
        SignatureDecl::operation("e_AB", [], Ty::sort("SiteAB")),
        SignatureDecl::operation("bond_AB", [], Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")])),
        SignatureDecl::operation("bond_C", [], Ty::tensor([Ty::sort("SiteAB"), Ty::sort("SiteC")])),
    ])
    .unwrap()
}

/// Signature for a toy model (emergent agent with directionality).
#[cfg(test)]
pub(crate) fn toy_signature_directionality() -> Signature {
    Signature::parse([
        SignatureDecl::sort("head"),
        SignatureDecl::sort("tail"),
        SignatureDecl::sort("Site_C"),
        SignatureDecl::sort("Site_ABh"),
        SignatureDecl::sort("Site_ABt"),
        SignatureDecl::operation("e_h", [], Ty::sort("head")),
        SignatureDecl::operation("e_t", [], Ty::sort("tail")),
        SignatureDecl::operation("e_C", [], Ty::sort("Site_C")),
        SignatureDecl::operation("e_ABh", [], Ty::sort("Site_ABh")),
        SignatureDecl::operation("e_ABt", [], Ty::sort("Site_ABt")),
        SignatureDecl::operation("bond_AB", [], Ty::tensor([Ty::sort("head"), Ty::sort("tail")])),
        SignatureDecl::operation(
            "bond_Ch",
            [],
            Ty::tensor([Ty::sort("Site_ABh"), Ty::sort("Site_C")]),
        ),
        SignatureDecl::operation(
            "bond_Ct",
            [],
            Ty::tensor([Ty::sort("Site_ABt"), Ty::sort("Site_C")]),
        ),
    ])
    .unwrap()
}

/// Signature for a toy model (phospho tyrosine).
#[cfg(test)]
pub(crate) fn toy_signature_phospho_tyrosine() -> Signature {
    Signature::parse([
        SignatureDecl::sort("Tyr"),
        SignatureDecl::sort("SH2"),
        SignatureDecl::sort("xTyr"),
        SignatureDecl::operation("e_sh2", [], Ty::sort("SH2")),
        SignatureDecl::operation("e_xtyr", [], Ty::sort("xTyr")),
        SignatureDecl::operation("u", [Ty::sort("xTyr")], Ty::sort("Tyr")),
        SignatureDecl::operation("p", [Ty::sort("xTyr")], Ty::sort("Tyr")),
        SignatureDecl::operation("bond", [], Ty::tensor([Ty::sort("SH2"), Ty::sort("xTyr")])),
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

        let expected = expect![[r#"
            #/ sorts:
            ReMonomer
            ReA
            ReB
            ReB1
            ReB2
            ReK
            SiteA
            SiteB
            Res
            #/ operations:
            iota_A : [ReA] → ReMonomer
            iota_B : [ReB] → ReMonomer
            iota_B1 : [ReB1] → ReB
            iota_B2 : [ReB2] → ReB
            iota_K : [ReK] → ReMonomer
            iota_SiteA : [SiteA] → ReMonomer
            iota_SiteB : [SiteB] → ReMonomer
            iota_Res : [Res] → ReMonomer
            phos : [] → Res
            unphos : [] → Res
            ground_A : [] → ReA
            ground_K : [] → ReK
            emptyA : [] → SiteA
            emptyB : [] → SiteB
            bond : [] → ⊗ [SiteA, SiteB]
            ground_B : [] → ReB
        "#]];
        expected.assert_eq(&toy_signature_species_granularity_1().to_string());

        let expected = expect![[r#"
            #/ sorts:
            ReMonomer
            ReA
            ReB
            ReB1
            ReB2
            ReK
            SiteA
            SiteB
            Res
            #/ operations:
            iota_A : [ReA] → ReMonomer
            iota_B : [ReB] → ReMonomer
            iota_B1 : [ReB1] → ReB
            iota_B2 : [ReB2] → ReB
            iota_K : [ReK] → ReMonomer
            iota_SiteA : [SiteA] → ReMonomer
            iota_SiteB : [SiteB] → ReMonomer
            iota_Res : [Res] → ReMonomer
            phos : [] → Res
            unphos : [] → Res
            ground_A : [] → ReA
            ground_K : [] → ReK
            emptyA : [] → SiteA
            emptyB : [] → SiteB
            bond : [] → ⊗ [SiteA, SiteB]
            ground_B1 : [] → ReB1
            ground_B2 : [] → ReB2
        "#]];
        expected.assert_eq(&toy_signature_species_granularity_2().to_string());

        let expected = expect![[r#"
            #/ sorts:
            SiteA
            SiteB
            SiteC
            SiteAB
            #/ operations:
            e_A : [] → SiteA
            e_B : [] → SiteB
            e_C : [] → SiteC
            e_AB : [] → SiteAB
            bond_AB : [] → ⊗ [SiteA, SiteB]
            bond_C : [] → ⊗ [SiteAB, SiteC]
        "#]];
        expected.assert_eq(&toy_signature_emergent_agent().to_string());

        let expected = expect![[r#"
            #/ sorts:
            head
            tail
            Site_C
            Site_ABh
            Site_ABt
            #/ operations:
            e_h : [] → head
            e_t : [] → tail
            e_C : [] → Site_C
            e_ABh : [] → Site_ABh
            e_ABt : [] → Site_ABt
            bond_AB : [] → ⊗ [head, tail]
            bond_Ch : [] → ⊗ [Site_ABh, Site_C]
            bond_Ct : [] → ⊗ [Site_ABt, Site_C]
        "#]];
        expected.assert_eq(&toy_signature_directionality().to_string());

        let expected = expect![[r#"
            #/ sorts:
            Tyr
            SH2
            xTyr
            #/ operations:
            e_sh2 : [] → SH2
            e_xtyr : [] → xTyr
            u : [xTyr] → Tyr
            p : [xTyr] → Tyr
            bond : [] → ⊗ [SH2, xTyr]
        "#]];
        expected.assert_eq(&toy_signature_phospho_tyrosine().to_string());
    }
}
