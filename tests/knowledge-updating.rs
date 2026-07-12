//! https://q.uiver.app/#q=WzAsNDIsWzEsMSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsMSwiU2l0ZUIiXSxbMiwwLCJUeUFnZW50Il0sWzYsMCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSwyLCJJIl0sWzMsMiwiSSJdLFs0LDAsIlNpdGVBIl0sWzUsMCwiU2l0ZUIiXSxbNywwLCJSZXMiXSxbNywyLCJJIl0sWzEsNSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsNSwiU2l0ZUIiXSxbMiw0LCJUeUFnZW50Il0sWzYsNCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSw1LCJTX2IiXSxbMyw1LCJTX2siXSxbNCw0LCJTaXRlQSJdLFs1LDQsIlNpdGVCIl0sWzcsNCwiUmVzIl0sWzcsNSwiU19wIl0sWzMsNiwiSSJdLFs1LDYsIkkiXSxbNyw2LCJJIl0sWzEsOSwiU2l0ZUEiXSxbMiw5LCJTaXRlQiJdLFsyLDgsIlR5QWdlbnQiXSxbOCw4LCJTaXRlQTEgXFxvdGltZXMgU2l0ZUIxIl0sWzYsMTAsIlNfYiJdLFs1LDgsIlNpdGVBIl0sWzYsOCwiU2l0ZUIiXSxbNiwxMSwiSSJdLFs0LDksIlNpdGVBMSJdLFs1LDksIlNpdGVBMiJdLFs2LDksIlNpdGVCMSJdLFs3LDksIlNpdGVCMiJdLFs5LDgsIlNpdGVBMiBcXG90aW1lcyBTaXRlQjIiXSxbOCwxMCwiU197YjF9Il0sWzksMTAsIlNfe2IyfSJdLFswLDEwLCJTaXRlQTEiXSxbMSwxMCwiU2l0ZUEyIl0sWzIsMTAsIlNpdGVCMSJdLFszLDEwLCJTaXRlQjIiXSxbNSwyLCJcXGlvdGFfSyIsMl0sWzQsNywiZW1wdHlfQiIsMV0sWzksOCwicGhvcyIsMCx7ImN1cnZlIjotMX1dLFs0LDYsImVtcHR5X0EiXSxbNCwzLCJib25kX3tBQn0iLDJdLFswLDIsIlxcaW90YV9BIiwyXSxbMSwyLCJcXGlvdGFfQiIsMl0sWzE1LDEyXSxbMTQsMTcsImVtcHR5X0IiLDFdLFsxOSwxOCwicCIsMCx7ImN1cnZlIjotMX1dLFsxNCwxNiwiZW1wdHlfQSJdLFsxNCwxMywiYm9uZF97QUJ9IiwyXSxbMTAsMTJdLFsxMSwxMl0sWzIwLDE1XSxbMjEsMTRdLFsyMiwxOV0sWzksOCwidW5waG9zIiwyLHsiY3VydmUiOjF9XSxbMTksMTgsInUiLDIseyJjdXJ2ZSI6MX1dLFsyMywyNV0sWzI0LDI1XSxbMzAsMjddLFszMSwyOF0sWzI3LDMxLCJlbXB0eV9BIl0sWzMyLDI4XSxbMjcsMzJdLFszNCwyOV0sWzI3LDMzLCJlbXB0eV9CIiwxXSxbMzMsMjldLFsyNywzNF0sWzM2LDI2XSxbMzcsMzVdLFszOCwyM10sWzM5LDIzXSxbNDAsMjRdLFs0MSwyNF1d

mod common;
use common::*;
use rulett::prelude::name;

/// Base signature.
fn main_signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyAgent"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        SignatureDecl::sort("SiteA1"),
        SignatureDecl::sort("SiteB1"),
        SignatureDecl::sort("SiteA2"),
        SignatureDecl::sort("SiteB2"),
        // Operations
        SignatureDecl::operation("i_A", [Ty::sort("SiteA")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_B", [Ty::sort("SiteB")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_A1", [Ty::sort("SiteA1")], Ty::sort("SiteA")),
        SignatureDecl::operation("i_A2", [Ty::sort("SiteA2")], Ty::sort("SiteA")),
        SignatureDecl::operation("i_B1", [Ty::sort("SiteB1")], Ty::sort("SiteB")),
        SignatureDecl::operation("i_B2", [Ty::sort("SiteB2")], Ty::sort("SiteB")),
    ])
    .unwrap()
}

/// Grounding signature.
fn grounding_signature_1() -> Signature {
    let d1 = [
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("SiteA1"),
        SignatureDecl::sort("SiteA2"),
        SignatureDecl::sort("SiteB1"),
        SignatureDecl::sort("SiteB2"),
        // Operations
        SignatureDecl::operation("empty_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("empty_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("empty_B1", [], Ty::sort("SiteB1")),
        SignatureDecl::operation("empty_B2", [], Ty::sort("SiteB2")),
        SignatureDecl::operation(
            "bond_1",
            [],
            Ty::tensor([Ty::sort("SiteA1"), Ty::sort("SiteB1")]),
        ),
        SignatureDecl::operation(
            "bond_2",
            [],
            Ty::tensor([Ty::sort("SiteA2"), Ty::sort("SiteB2")]),
        ),
    ];
    Signature::parse(d1).unwrap()
}

/// Full signature.
fn signature() -> Signature {
    let sig1 = main_signature();
    let sig2 = grounding_signature_1();
    merge_signatures(&[sig1, sig2])
}

// Declares Model.
fn model_decl() -> [ModelDecl; 3] {
    use crate::surface::*;
    // Define agent
    let agent = ModelDecl::agent("Agent", [ObTm::var("a")], [Ty::sort("TyAgent")]);

    // Define patterns
    let a1 = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("i_A1", [MorTm::var("s1")])])]);
    let a2 = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("i_A2", [MorTm::var("s2")])])]);
    let b1 = PatTm::res("Agent", [MorTm::app("i_B", [MorTm::app("i_B1", [MorTm::var("s3")])])]);
    let b2 = PatTm::res("Agent", [MorTm::app("i_B", [MorTm::app("i_B2", [MorTm::var("s4")])])]);
    let a1_free = a1.subst(&mut vec![(name("s1"), MorTm::app("empty_A1", []))]);
    let a2_free = a2.subst(&mut vec![(name("s2"), MorTm::app("empty_A2", []))]);
    let b1_free = b1.subst(&mut vec![(name("s3"), MorTm::app("empty_B1", []))]);
    let b2_free = a1.subst(&mut vec![(name("s4"), MorTm::app("empty_B2", []))]);
    let a1b1 = PatTm::let_(
        [ObTm::var("s1"), ObTm::var("s3")],
        [MorTm::var("bond_1")],
        PatTm::tensor([a1, b1]),
    );
    let a2b2 = PatTm::let_(
        [ObTm::var("s2"), ObTm::var("s4")],
        [MorTm::var("bond_2")],
        PatTm::tensor([a2, b2]),
    );
    // Define rules
    let bond_a1b1 = ModelDecl::rule("bond_A1B1", [], [], PatTm::tensor([a1_free, b1_free]), a1b1);
    let bond_a2b2 = ModelDecl::rule("bond_A2B2", [], [], PatTm::tensor([a2_free, b2_free]), a2b2);
    [agent, bond_a1b1, bond_a2b2]
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
        SiteA1
        SiteB1
        SiteA2
        SiteB2
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_B1 : [SiteB1] → SiteB
        i_B2 : [SiteB2] → SiteB
        empty_A1 : [] → SiteA1
        empty_A2 : [] → SiteA2
        empty_B1 : [] → SiteB1
        empty_B2 : [] → SiteB2
        bond_1 : [] → ⊗ [SiteA1, SiteB1]
        bond_2 : [] → ⊗ [SiteA2, SiteB2]
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
        SiteA1
        SiteB1
        SiteA2
        SiteB2
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_B1 : [SiteB1] → SiteB
        i_B2 : [SiteB2] → SiteB
        empty_A1 : [] → SiteA1
        empty_A2 : [] → SiteA2
        empty_B1 : [] → SiteB1
        empty_B2 : [] → SiteB2
        bond_1 : [] → ⊗ [SiteA1, SiteB1]
        bond_2 : [] → ⊗ [SiteA2, SiteB2]
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [] : [] ⊢
          bond_A1B1 []
            : (Agent [i_A [i_A1 [empty_A1 []]]], Agent [i_B [i_B1 [empty_B1 []]]])
            → let [bond_1] in (Agent [i_A [i_A1 [0.0]]], Agent [i_B [i_B1 [0.1]]])
        [] : [] ⊢
          bond_A2B2 []
            : (Agent [i_A [i_A2 [empty_A2 []]]], Agent [i_A [i_A1 [s1]]])
            → let [bond_2] in (Agent [i_A [i_A2 [0.0]]], Agent [i_B [i_B2 [0.1]]])
    "#]];
    expected.assert_eq(&model().to_string());
}

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [i_A1 [empty_A1 []]]]
        Agent [i_A [i_A2 [empty_A2 []]]]
        Agent [i_B [i_B1 [empty_B1 []]]]
        Agent [i_B [i_B2 [empty_B2 []]]]
        let bond_1 [] in (Agent [i_A [i_A1 [0.0]]], Agent [i_B [i_B1 [0.1]]])
        let bond_2 [] in (Agent [i_A [i_A2 [0.0]]], Agent [i_B [i_B2 [0.1]]])
        let bond_1 [] in (Agent [i_B [i_B1 [0.1]]], Agent [i_A [i_A1 [0.0]]])
        let bond_2 [] in (Agent [i_B [i_B2 [0.1]]], Agent [i_A [i_A2 [0.0]]])"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bond_A1B1 []
          : (Agent [i_A [i_A1 [empty_A1 []]]], Agent [i_B [i_B1 [empty_B1 []]]])
          → let [bond_1] in (Agent [i_A [i_A1 [0.0]]], Agent [i_B [i_B1 [0.1]]])
        bond_A2B2 []
          : (Agent [i_A [i_A2 [empty_A2 []]]], Agent [i_A [i_A1 [s1]]])
          → let [bond_2] in (Agent [i_A [i_A2 [0.0]]], Agent [i_B [i_B2 [0.1]]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
