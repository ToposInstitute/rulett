/// Eucaryotic cells contain membrane-enclosed compartments.
/// The concentrations of chemical species may differ between compartments.
/// To describe this circumstance, we can introduce a preorder of compartments
/// to our typing category.
///
mod common;
use common::*;

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

/// Signature for a toy model (variant 2).
fn toy_signature_v2() -> Signature {
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

fn signature_localization() -> Signature {
    Signature::parse([
        SignatureDecl::sort("LocCell"),
        SignatureDecl::sort("LocCyt"), // Cytoplasm
        SignatureDecl::sort("LocNuc"), // Nucleus
        SignatureDecl::operation("cyt", [Ty::sort("LocCyt")], Ty::sort("LocCell")),
        SignatureDecl::operation("nuc", [Ty::sort("LocNuc")], Ty::sort("LocCell")),
    ])
    .unwrap()
}

fn ground_localization() -> Signature {
    Signature::parse([
        SignatureDecl::sort("LocCyt"),                            // Cytoplasm
        SignatureDecl::sort("LocNuc"),                            // Nucleus
        SignatureDecl::operation("!cyt", [], Ty::sort("LocCyt")), // Maybe it would be a good convention to use `!` or some other indication to label morphisms from I.
        SignatureDecl::operation("!nuc", [], Ty::sort("LocNuc")),
    ])
    .unwrap()
}

fn model_decl() -> [ModelDecl; 6] {
    use crate::surface::*;
    // TODO: make location mandatory for this setting (perhaps by adding a loc field to agents?)
    [
        ModelDecl::agent(
            "A",
            [ObTm::var("r"), ObTm::var("s"), ObTm::var("l")],
            [Ty::sort("Res"), Ty::sort("SiteA"), Ty::sort("LocCell")],
        ),
        ModelDecl::agent(
            "B",
            [ObTm::var("s"), ObTm::var("l")],
            [Ty::sort("SiteB"), Ty::sort("LocCell")],
        ),
        ModelDecl::agent("K", [ObTm::var("l")], [Ty::sort("LocCell")]),
        // Nuclear import of phosphorylated A
        ModelDecl::rule(
            "nuclear_import_phospho_A",
            [ObTm::var("s")],
            [Ty::sort("SiteA")],
            PatTm::res("A", [MorTm::var("p"), MorTm::var("s"), MorTm::app("!cyt", [])]),
            PatTm::res("A", [MorTm::var("p"), MorTm::var("s"), MorTm::app("!nuc", [])]),
        ),
        // Non-transport rules are location agnostic, but require all reactants to be in the same location
        ModelDecl::rule(
            "bondAB",
            [ObTm::var("r"), ObTm::var("l")],
            [Ty::sort("Res"), Ty::sort("LocCell")],
            PatTm::tensor([
                PatTm::res("A", [MorTm::var("r"), MorTm::app("emptyA", []), MorTm::var("l")]),
                PatTm::res("B", [MorTm::app("emptyB", []), MorTm::var("l")]),
            ]),
            PatTm::let_(
                ObTm::tensor([ObTm::var("s1"), ObTm::var("s2")]),
                MorTm::app("bond", []),
                PatTm::tensor([
                    PatTm::res("A", [MorTm::var("r"), MorTm::var("s1"), MorTm::var("l")]),
                    PatTm::res("B", [MorTm::var("s2"), MorTm::var("l")]),
                ]),
            ),
        ),
        ModelDecl::rule(
            "phosphorylate",
            [ObTm::var("s"), ObTm::var("l")],
            [Ty::sort("SiteA"), Ty::sort("LocCell")],
            PatTm::tensor([
                PatTm::res("A", [MorTm::app("unphos", []), MorTm::var("s"), MorTm::var("l")]),
                PatTm::res("K", [MorTm::var("l")]),
            ]),
            PatTm::tensor([
                PatTm::res("A", [MorTm::app("phos", []), MorTm::var("s"), MorTm::var("l")]),
                PatTm::res("K", [MorTm::var("l")]),
            ]),
        ),
    ]
}

fn model() -> Model {
    let decls = model_decl();
    Model::parse(signature(), decls).unwrap()
}

use expect_test::expect;

/// Signature for toy_model_v2 with localization
fn signature() -> Signature {
    let sig1 = toy_signature_v2();
    let sig2 = signature_localization();
    let sig3 = ground_localization();
    merge_signatures(&[sig1, sig2, sig3])
}

#[test]
fn parse_signature() {
    let expected = expect![[r#"
            #/ sorts:
            Res
            SiteA
            SiteB
            LocCell
            LocCyt
            LocNuc
            #/ operations:
            unphos : [] → Res
            phos : [] → Res
            emptyA : [] → SiteA
            emptyB : [] → SiteB
            bond : [] → ⊗ [SiteA, SiteB]
            cyt : [LocCyt] → LocCell
            nuc : [LocNuc] → LocCell
            !cyt : [] → LocCyt
            !nuc : [] → LocNuc
        "#]];
    expected.assert_eq(&signature().to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        Res
        SiteA
        SiteB
        LocCell
        LocCyt
        LocNuc
        #/ operations:
        unphos : [] → Res
        phos : [] → Res
        emptyA : [] → SiteA
        emptyB : [] → SiteB
        bond : [] → ⊗ [SiteA, SiteB]
        cyt : [LocCyt] → LocCell
        nuc : [LocNuc] → LocCell
        !cyt : [] → LocCyt
        !nuc : [] → LocNuc
        #/ agents:
        [r, s, l] : [Res, SiteA, LocCell] ⊢ A [r, s, l]
        [s, l] : [SiteB, LocCell] ⊢ B [s, l]
        [l] : [LocCell] ⊢ K [l]
        #/ rules:
        [s] : [SiteA] ⊢
          nuclear_import_phospho_A [s] : A [p, s, !cyt []] → A [p, s, !nuc []]
        [r, l] : [Res, LocCell] ⊢
          bondAB [r, l]
            : (A [r, emptyA [], l], B [emptyB [], l])
            → let bond [] in (A [r, 0.0, l], B [0.1, l])
        [s, l] : [SiteA, LocCell] ⊢
          phosphorylate [s, l]
            : (A [unphos [], s, l], K [l])
            → (A [phos [], s, l], K [l])
    "#]];
    expected.assert_eq(&model().to_string());
}
