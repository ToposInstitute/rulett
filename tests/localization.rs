/// Eucaryotic cells contain membrane-enclosed compartments.
/// The concentrations of chemical species may differ between compartments.
/// To describe this circumstance, we can introduce a preorder of compartments
/// to our typing category.
///
mod common;
use common::*;

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
    // TODO: make location mandatory for this setting (perhaps by adding a loctm and locty fields to agents?)
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
        // @Evan: note that `l` appears multiple times here, cause K cannot phosphorylate A if they are in different compartments.
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

// use super::{super::model, super::theory::*, *};

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model();
    let generator = NetGenerator::new(&model);

    // TODO: Think about whether you want to have multi-compartment species here (e.g.: let bond [] in (A [unphos [], 0.0, cyt [!cyt []]], B [0.1, nuc [!nuc []]])).
    let species = expect![[r#"
        A [unphos [], emptyA [], cyt [!cyt []]]
        A [unphos [], emptyA [], nuc [!nuc []]]
        A [phos [], emptyA [], cyt [!cyt []]]
        A [phos [], emptyA [], nuc [!nuc []]]
        B [emptyB [], cyt [!cyt []]]
        B [emptyB [], nuc [!nuc []]]
        K [cyt [!cyt []]]
        K [nuc [!nuc []]]
        let bond [] in (A [unphos [], 0.0, cyt [!cyt []]], B [0.1, cyt [!cyt []]])
        let bond [] in (A [unphos [], 0.0, cyt [!cyt []]], B [0.1, nuc [!nuc []]])
        let bond [] in (A [unphos [], 0.0, nuc [!nuc []]], B [0.1, cyt [!cyt []]])
        let bond [] in (A [unphos [], 0.0, nuc [!nuc []]], B [0.1, nuc [!nuc []]])
        let bond [] in (A [phos [], 0.0, cyt [!cyt []]], B [0.1, cyt [!cyt []]])
        let bond [] in (A [phos [], 0.0, cyt [!cyt []]], B [0.1, nuc [!nuc []]])
        let bond [] in (A [phos [], 0.0, nuc [!nuc []]], B [0.1, cyt [!cyt []]])
        let bond [] in (A [phos [], 0.0, nuc [!nuc []]], B [0.1, nuc [!nuc []]])"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![[r#"
        nuclear_import_phospho_A [emptyA []]
          : A [p, emptyA [], !cyt []]
          → A [p, emptyA [], !nuc []]
        bondAB [unphos [], cyt [!cyt []]]
          : (A [unphos [], emptyA [], cyt [!cyt []]], B [emptyB [], cyt [!cyt []]])
          → let bond [] in (A [unphos [], 0.0, cyt [!cyt []]], B [0.1, cyt [!cyt []]])
        bondAB [unphos [], nuc [!nuc []]]
          : (A [unphos [], emptyA [], nuc [!nuc []]], B [emptyB [], nuc [!nuc []]])
          → let bond [] in (A [unphos [], 0.0, nuc [!nuc []]], B [0.1, nuc [!nuc []]])
        bondAB [phos [], cyt [!cyt []]]
          : (A [phos [], emptyA [], cyt [!cyt []]], B [emptyB [], cyt [!cyt []]])
          → let bond [] in (A [phos [], 0.0, cyt [!cyt []]], B [0.1, cyt [!cyt []]])
        bondAB [phos [], nuc [!nuc []]]
          : (A [phos [], emptyA [], nuc [!nuc []]], B [emptyB [], nuc [!nuc []]])
          → let bond [] in (A [phos [], 0.0, nuc [!nuc []]], B [0.1, nuc [!nuc []]])
        phosphorylate [emptyA [], cyt [!cyt []]]
          : (A [unphos [], emptyA [], cyt [!cyt []]], K [cyt [!cyt []]])
          → (A [phos [], emptyA [], cyt [!cyt []]], K [cyt [!cyt []]])
        phosphorylate [emptyA [], nuc [!nuc []]]
          : (A [unphos [], emptyA [], nuc [!nuc []]], K [nuc [!nuc []]])
          → (A [phos [], emptyA [], nuc [!nuc []]], K [nuc [!nuc []]])
        let bond [] in (B [0.1, cyt [!cyt []]], nuclear_import_phospho_A [0.0])
          : let bond [] in (B [0.1, cyt [!cyt []]], A [p, 0.0, !cyt []])
          → let bond [] in (B [0.1, cyt [!cyt []]], A [p, 0.0, !nuc []])
        let bond [] in (B [0.1, nuc [!nuc []]], nuclear_import_phospho_A [0.0])
          : let bond [] in (B [0.1, nuc [!nuc []]], A [p, 0.0, !cyt []])
          → let bond [] in (B [0.1, nuc [!nuc []]], A [p, 0.0, !nuc []])
        let bond [] in (B [0.1, cyt [!cyt []]], phosphorylate [0.0, cyt [!cyt []]])
          : let bond [] in
            (
              B [0.1, cyt [!cyt []]],
              (A [unphos [], 0.0, cyt [!cyt []]], K [cyt [!cyt []]])
            )
          → let bond [] in
            (
              B [0.1, cyt [!cyt []]],
              (A [phos [], 0.0, cyt [!cyt []]], K [cyt [!cyt []]])
            )
        let bond [] in (B [0.1, cyt [!cyt []]], phosphorylate [0.0, nuc [!nuc []]])
          : let bond [] in
            (
              B [0.1, cyt [!cyt []]],
              (A [unphos [], 0.0, nuc [!nuc []]], K [nuc [!nuc []]])
            )
          → let bond [] in
            (
              B [0.1, cyt [!cyt []]],
              (A [phos [], 0.0, nuc [!nuc []]], K [nuc [!nuc []]])
            )
        let bond [] in (B [0.1, nuc [!nuc []]], phosphorylate [0.0, cyt [!cyt []]])
          : let bond [] in
            (
              B [0.1, nuc [!nuc []]],
              (A [unphos [], 0.0, cyt [!cyt []]], K [cyt [!cyt []]])
            )
          → let bond [] in
            (
              B [0.1, nuc [!nuc []]],
              (A [phos [], 0.0, cyt [!cyt []]], K [cyt [!cyt []]])
            )
        let bond [] in (B [0.1, nuc [!nuc []]], phosphorylate [0.0, nuc [!nuc []]])
          : let bond [] in
            (
              B [0.1, nuc [!nuc []]],
              (A [unphos [], 0.0, nuc [!nuc []]], K [nuc [!nuc []]])
            )
          → let bond [] in
            (
              B [0.1, nuc [!nuc []]],
              (A [phos [], 0.0, nuc [!nuc []]], K [nuc [!nuc []]])
            )"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
