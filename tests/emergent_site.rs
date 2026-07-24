//! This example illustrates how bonding of A to B creates a new, "emergent" site that can bind to another agent C.
//! Additionally, we show "molecular coarse graining": we replace the "atomic" SiteA and SiteB (think monomers) with a
//! by removing the morphisms i_A: SiteA -> TyAgent and i_B: SiteB -> TyAgent via a morphism i_AB: SiteAB -> TyAgent.
//!
//! https://q.uiver.app/#q=WzAsNyxbMSwwLCJTaXRlQSBcXG90aW1lcyBTaXRlQiJdLFsxLDEsIlNpdGVBQiJdLFsxLDIsIkkiXSxbMCwyLCJTaXRlQSJdLFsyLDIsIlNpdGVCIl0sWzMsMCwiU2l0ZUFCIFxcb3RpbWVzIFNpdGVDIl0sWzMsMiwiSSJdLFsxLDAsImJvbmRfe0F0b0J9Il0sWzIsMSwiZW1wdHlfe0FCfSJdLFsyLDMsImVtcHR5X0EiLDJdLFsyLDQsImVtcHR5X0IiLDJdLFs2LDUsImJvbmRfe0FCdG9DfSIsMl1d

mod common;
use common::*;
use rulett::prelude::name;

/// Main signature.
fn signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyAgent"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        SignatureDecl::sort("SiteAB"),
        SignatureDecl::sort("SiteC"),
        // Operations
        SignatureDecl::operation("i_A", [Ty::sort("SiteA")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_B", [Ty::sort("SiteB")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_C", [Ty::sort("SiteC")], Ty::sort("TyAgent")),
        SignatureDecl::operation("empty_A", [], Ty::sort("SiteA")),
        SignatureDecl::operation("empty_B", [], Ty::sort("SiteB")),
        SignatureDecl::operation("empty_C", [], Ty::sort("SiteC")),
        SignatureDecl::operation("empty_AB", [], Ty::sort("SiteAB")),
        SignatureDecl::operation(
            "bond_AtoB",
            [Ty::sort("SiteAB")],
            Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")]),
        ),
        SignatureDecl::operation(
            "bond_ABtoC",
            [],
            Ty::tensor([Ty::sort("SiteC"), Ty::sort("SiteAB")]),
        ),
    ])
    .unwrap()
}

/// Derlares Model.
fn model_decl() -> [ModelDecl; 3] {
    use crate::surface::*;

    // Define agent
    let agent = ModelDecl::agent("Agent", [ObTm::var("a")], [Ty::sort("TyAgent")]);

    // Define patterns
    let a = PatTm::res("TyAgent", [MorTm::app("i_A", [MorTm::var("s1")])]);
    let b = PatTm::res("TyAgent", [MorTm::app("i_B", [MorTm::var("s2")])]);
    let c = PatTm::res("TyAgent", [MorTm::app("i_C", [MorTm::var("s3")])]);
    let a_free = a.subst(&mut vec![(name("s1"), MorTm::app("empty_A", []))]);
    let b_free = b.subst(&mut vec![(name("s2"), MorTm::app("empty_B", []))]);
    let c_free = c.subst(&mut vec![(name("s3"), MorTm::app("empty_C", []))]);

    let ab_free = PatTm::let_(
        [ObTm::var("s1"), ObTm::var("s2")],
        [MorTm::app("bond_AtoB", [MorTm::app("empty_AB", [])])],
        PatTm::tensor([a.clone(), b.clone()]),
    );
    let ab = PatTm::let_(
        [ObTm::var("s1"), ObTm::var("s2")],
        [MorTm::app("bond_AtoB", [MorTm::var("s4")])],
        PatTm::tensor([a, b]),
    );

    let abc = PatTm::let_(
        [ObTm::var("s3"), ObTm::var("s4")],
        MorTm::app("bond_ABtoC", []),
        PatTm::tensor([c, ab]),
    );

    // Define rules
    let bond_ab =
        ModelDecl::rule("bond_AB", [], [], PatTm::tensor([a_free, b_free]), ab_free.clone());
    let bond_c = ModelDecl::rule("bond_ABC", [], [], PatTm::tensor([ab_free, c_free]), abc);
    [agent, bond_ab, bond_c]
}

// Generates Model.
fn model() -> Model {
    let decls = model_decl();
    Model::parse(signature(), decls).unwrap()
}

#[test]
fn parse_signature() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteB
        SiteAB
        SiteC
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_C : [SiteC] → TyAgent
        empty_A : [] → SiteA
        empty_B : [] → SiteB
        empty_C : [] → SiteC
        empty_AB : [] → SiteAB
        bond_AtoB : [SiteAB] → ⊗ [SiteA, SiteB]
        bond_ABtoC : [] → ⊗ [SiteC, SiteAB]
    "#]];
    expected.assert_eq(&signature().to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteB
        SiteAB
        SiteC
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_C : [SiteC] → TyAgent
        empty_A : [] → SiteA
        empty_B : [] → SiteB
        empty_C : [] → SiteC
        empty_AB : [] → SiteAB
        bond_AtoB : [SiteAB] → ⊗ [SiteA, SiteB]
        bond_ABtoC : [] → ⊗ [SiteC, SiteAB]
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [] : [] ⊢
          bond_AB []
            : (TyAgent [i_A [empty_A []]], TyAgent [i_B [empty_B []]])
            → let [bond_AtoB [empty_AB []]] in
              (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
        [] : [] ⊢
          bond_ABC []
            : (
              let [bond_AtoB [empty_AB []]] in
                (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]]),
              TyAgent [i_C [empty_C []]]
            )
            → let bond_ABtoC [] in
              (
                TyAgent [i_C [0.0]],
                let [bond_AtoB [0.1]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
              )
    "#]];
    expected.assert_eq(&model().to_string());
}

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [empty_A []]]
        Agent [i_B [empty_B []]]
        Agent [i_C [empty_C []]]
        let bond_AtoB [empty_AB []] in (Agent [i_A [0.0]], Agent [i_B [0.1]])
        let bond_AtoB [empty_AB []] in (Agent [i_B [0.1]], Agent [i_A [0.0]])
        let bond_ABtoC [] in
          let bond_AtoB [0.1] in
            (Agent [i_A [0.0]], Agent [i_B [0.1]], Agent [i_C [1.0]])
        let bond_ABtoC [] in
          let bond_AtoB [0.1] in
            (Agent [i_A [0.0]], Agent [i_C [1.0]], Agent [i_B [0.1]])
        let bond_ABtoC [] in
          let bond_AtoB [0.1] in
            (Agent [i_B [0.1]], Agent [i_A [0.0]], Agent [i_C [1.0]])
        let bond_ABtoC [] in
          let bond_AtoB [0.1] in
            (Agent [i_B [0.1]], Agent [i_C [1.0]], Agent [i_A [0.0]])
        let bond_ABtoC [] in
          let bond_AtoB [0.1] in
            (Agent [i_C [1.0]], Agent [i_A [0.0]], Agent [i_B [0.1]])
        let bond_ABtoC [] in
          let bond_AtoB [0.1] in
            (Agent [i_C [1.0]], Agent [i_B [0.1]], Agent [i_A [0.0]])"#]];
    species.assert_eq(&generator.species(3).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bond_AB []
          : (TyAgent [i_A [empty_A []]], TyAgent [i_B [empty_B []]])
          → let [bond_AtoB [empty_AB []]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
        bond_ABC []
          : (
            let [bond_AtoB [empty_AB []]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]]),
            TyAgent [i_C [empty_C []]]
          )
          → let bond_ABtoC [] in
            (
              TyAgent [i_C [0.0]],
              let [bond_AtoB [0.1]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
            )"#]];
    transitions.assert_eq(&generator.transitions(3).join("\n"));
}

// ==========================================
// --- Molecular coarse graining
// ==========================================

fn signature_mcg() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyAgent"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        SignatureDecl::sort("SiteAB"),
        SignatureDecl::sort("SiteC"),
        // Operations
        SignatureDecl::operation("i_AB", [Ty::sort("SiteAB")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_C", [Ty::sort("SiteC")], Ty::sort("TyAgent")),
        SignatureDecl::operation("empty_A", [], Ty::sort("SiteA")),
        SignatureDecl::operation("empty_B", [], Ty::sort("SiteB")),
        SignatureDecl::operation("empty_C", [], Ty::sort("SiteC")),
        SignatureDecl::operation("empty_AB", [], Ty::sort("SiteAB")),
        SignatureDecl::operation(
            "bond_AtoB",
            [Ty::sort("SiteAB")],
            Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")]),
        ),
        SignatureDecl::operation(
            "bond_ABtoC",
            [],
            Ty::tensor([Ty::sort("SiteAB"), Ty::sort("SiteC")]),
        ),
    ])
    .unwrap()
}

// Generates Model.
fn model_mcg() -> Model {
    let decls = model_decl();
    Model::parse(signature_mcg(), decls).unwrap()
}

#[test]
fn parse_signature_mcg() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteB
        SiteAB
        SiteC
        #/ operations:
        i_AB : [SiteAB] → TyAgent
        i_C : [SiteC] → TyAgent
        empty_A : [] → SiteA
        empty_B : [] → SiteB
        empty_C : [] → SiteC
        empty_AB : [] → SiteAB
        bond_AtoB : [SiteAB] → ⊗ [SiteA, SiteB]
        bond_ABtoC : [] → ⊗ [SiteAB, SiteC]
    "#]];
    expected.assert_eq(&signature_mcg().to_string());
}

#[test]
fn parse_model_mcg() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteB
        SiteAB
        SiteC
        #/ operations:
        i_AB : [SiteAB] → TyAgent
        i_C : [SiteC] → TyAgent
        empty_A : [] → SiteA
        empty_B : [] → SiteB
        empty_C : [] → SiteC
        empty_AB : [] → SiteAB
        bond_AtoB : [SiteAB] → ⊗ [SiteA, SiteB]
        bond_ABtoC : [] → ⊗ [SiteAB, SiteC]
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [] : [] ⊢
          bond_AB []
            : (TyAgent [i_A [empty_A []]], TyAgent [i_B [empty_B []]])
            → let [bond_AtoB [empty_AB []]] in
              (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
        [] : [] ⊢
          bond_ABC []
            : (
              let [bond_AtoB [empty_AB []]] in
                (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]]),
              TyAgent [i_C [empty_C []]]
            )
            → let bond_ABtoC [] in
              (
                TyAgent [i_C [0.0]],
                let [bond_AtoB [0.1]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
              )
    "#]];
    expected.assert_eq(&model_mcg().to_string());
}

#[test]
fn generate_network_mcg() {
    use itertools::Itertools;
    let model = model_mcg();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_AB [empty_AB []]]
        Agent [i_C [empty_C []]]
        let bond_ABtoC [] in (Agent [i_AB [0.0]], Agent [i_C [0.1]])
        let bond_ABtoC [] in (Agent [i_C [0.1]], Agent [i_AB [0.0]])"#]];
    species.assert_eq(&generator.species(3).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bond_AB []
          : (TyAgent [i_A [empty_A []]], TyAgent [i_B [empty_B []]])
          → let [bond_AtoB [empty_AB []]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
        bond_ABC []
          : (
            let [bond_AtoB [empty_AB []]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]]),
            TyAgent [i_C [empty_C []]]
          )
          → let bond_ABtoC [] in
            (
              TyAgent [i_C [0.0]],
              let [bond_AtoB [0.1]] in (TyAgent [i_A [0.0]], TyAgent [i_B [0.1]])
            )"#]];
    transitions.assert_eq(&generator.transitions(3).join("\n"));
}
