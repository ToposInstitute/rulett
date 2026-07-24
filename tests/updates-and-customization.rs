//! This example shows how to
//! - generalize knowledge (shift a rule to a higher level of abstraction)
//! - coarse grain models through coproducts (use point to higher level rules)
//! - coarse grain models through products (not working yet - requires equality contraints)
//!
//! https://q.uiver.app/#q=WzAsNDIsWzEsMSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsMSwiU2l0ZUIiXSxbMiwwLCJUeUFnZW50Il0sWzYsMCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSwyLCJJIl0sWzMsMiwiSSJdLFs0LDAsIlNpdGVBIl0sWzUsMCwiU2l0ZUIiXSxbNywwLCJSZXMiXSxbNywyLCJJIl0sWzEsNSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsNSwiU2l0ZUIiXSxbMiw0LCJUeUFnZW50Il0sWzYsNCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSw1LCJTX2IiXSxbMyw1LCJTX2siXSxbNCw0LCJTaXRlQSJdLFs1LDQsIlNpdGVCIl0sWzcsNCwiUmVzIl0sWzcsNSwiU19wIl0sWzMsNiwiSSJdLFs1LDYsIkkiXSxbNyw2LCJJIl0sWzEsOSwiU2l0ZUEiXSxbMiw5LCJTaXRlQiJdLFsyLDgsIlR5QWdlbnQiXSxbOCw4LCJTaXRlQTEgXFxvdGltZXMgU2l0ZUIxIl0sWzYsMTAsIlNfYiJdLFs1LDgsIlNpdGVBIl0sWzYsOCwiU2l0ZUIiXSxbNiwxMSwiSSJdLFs0LDksIlNpdGVBMSJdLFs1LDksIlNpdGVBMiJdLFs2LDksIlNpdGVCMSJdLFs3LDksIlNpdGVCMiJdLFs5LDgsIlNpdGVBMiBcXG90aW1lcyBTaXRlQjIiXSxbOCwxMCwiU197YjF9Il0sWzksMTAsIlNfe2IyfSJdLFswLDEwLCJTaXRlQTEiXSxbMSwxMCwiU2l0ZUEyIl0sWzIsMTAsIlNpdGVCMSJdLFszLDEwLCJTaXRlQjIiXSxbNSwyLCJcXGlvdGFfSyIsMl0sWzQsNywiZW1wdHlfQiIsMV0sWzksOCwicGhvcyIsMCx7ImN1cnZlIjotMX1dLFs0LDYsImVtcHR5X0EiXSxbNCwzLCJib25kX3tBQn0iLDJdLFswLDIsIlxcaW90YV9BIiwyXSxbMSwyLCJcXGlvdGFfQiIsMl0sWzE1LDEyXSxbMTQsMTcsImVtcHR5X0IiLDFdLFsxOSwxOCwicCIsMCx7ImN1cnZlIjotMX1dLFsxNCwxNiwiZW1wdHlfQSJdLFsxNCwxMywiYm9uZF97QUJ9IiwyXSxbMTAsMTJdLFsxMSwxMl0sWzIwLDE1XSxbMjEsMTRdLFsyMiwxOV0sWzksOCwidW5waG9zIiwyLHsiY3VydmUiOjF9XSxbMTksMTgsInUiLDIseyJjdXJ2ZSI6MX1dLFsyMywyNV0sWzI0LDI1XSxbMzAsMjddLFszMSwyOF0sWzI3LDMxLCJlbXB0eV9BIl0sWzMyLDI4XSxbMjcsMzJdLFszNCwyOV0sWzI3LDMzLCJlbXB0eV9CIiwxXSxbMzMsMjldLFsyNywzNF0sWzM2LDI2XSxbMzcsMzVdLFszOCwyM10sWzM5LDIzXSxbNDAsMjRdLFs0MSwyNF1d

#[allow(dead_code)]
mod common;
use common::*;
use rulett::prelude::name;

// ==========================================
// --- Basic case ---
// ==========================================

/// Base signature.
fn main_signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyAgent"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteK"),
        SignatureDecl::sort("SiteA1"),
        SignatureDecl::sort("SiteK1"),
        SignatureDecl::sort("SiteA2"),
        SignatureDecl::sort("SiteK2"),
        // Operations
        SignatureDecl::operation("i_A", [Ty::sort("SiteA")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_K", [Ty::sort("SiteK")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_A1", [Ty::sort("SiteA1")], Ty::sort("SiteA")),
        SignatureDecl::operation("i_A2", [Ty::sort("SiteA2")], Ty::sort("SiteA")),
        SignatureDecl::operation("i_K1", [Ty::sort("SiteK1")], Ty::sort("SiteK")),
        SignatureDecl::operation("i_K2", [Ty::sort("SiteK2")], Ty::sort("SiteK")),
    ])
    .unwrap()
}

/// Grounding signature
fn grounding_signature() -> Signature {
    let d = [
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteA1"),
        SignatureDecl::sort("SiteA2"),
        SignatureDecl::sort("SiteK1"),
        SignatureDecl::sort("SiteK2"),
        // Operations
        SignatureDecl::operation("unphos_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("unphos_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("phos_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("phos_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("!K1", [], Ty::sort("SiteK1")),
        SignatureDecl::operation("!K2", [], Ty::sort("SiteK2")),
    ];
    Signature::parse(d).unwrap()
}

/// Full signature.
fn signature() -> Signature {
    let sig1 = main_signature();
    let sig2 = grounding_signature();
    merge_signatures(&[sig1, sig2])
}

/// Declares Model.
fn model_decl() -> [ModelDecl; 3] {
    use crate::surface::*;
    // Define agent
    let agent = ModelDecl::agent("Agent", [ObTm::var("a")], [Ty::sort("TyAgent")]);

    // Define patterns
    let a1 = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("i_A1", [MorTm::var("s1")])])]);
    let a2 = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("i_A2", [MorTm::var("s2")])])]);
    let k1 = PatTm::res("Agent", [MorTm::app("i_K", [MorTm::app("i_K1", [MorTm::var("k1")])])]);
    let k2 = PatTm::res("Agent", [MorTm::app("i_K", [MorTm::app("i_K2", [MorTm::var("k2")])])]);
    let a1_unphos = a1.subst(&mut vec![(name("s1"), MorTm::app("unphos_A1", []))]);
    let a2_unphos = a2.subst(&mut vec![(name("s2"), MorTm::app("unphos_A2", []))]);
    let a1_phos = a1.subst(&mut vec![(name("s1"), MorTm::app("phos_A1", []))]);
    let a2_phos = a2.subst(&mut vec![(name("s2"), MorTm::app("phos_A2", []))]);
    // Define rules
    let phosphorylate_1 = ModelDecl::rule(
        "phosphorylate_1",
        [ObTm::var("k1")],
        [Ty::sort("SiteK1")],
        PatTm::tensor([a1_unphos, k1.clone()]),
        PatTm::tensor([a1_phos, k1]),
    );
    let phosphorylate_2 = ModelDecl::rule(
        "phosphorylate_2",
        [ObTm::var("k2")],
        [Ty::sort("SiteK2")],
        PatTm::tensor([a2_unphos, k2.clone()]),
        PatTm::tensor([a2_phos, k2]),
    );
    [agent, phosphorylate_1, phosphorylate_2]
}

/// Generates Model.
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
        SiteK
        SiteA1
        SiteK1
        SiteA2
        SiteK2
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_K : [SiteK] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_K1 : [SiteK1] → SiteK
        i_K2 : [SiteK2] → SiteK
        unphos_A1 : [] → SiteA1
        unphos_A2 : [] → SiteA2
        phos_A1 : [] → SiteA1
        phos_A2 : [] → SiteA2
        !K1 : [] → SiteK1
        !K2 : [] → SiteK2
    "#]];
    expected.assert_eq(&signature().to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteK
        SiteA1
        SiteK1
        SiteA2
        SiteK2
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_K : [SiteK] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_K1 : [SiteK1] → SiteK
        i_K2 : [SiteK2] → SiteK
        unphos_A1 : [] → SiteA1
        unphos_A2 : [] → SiteA2
        phos_A1 : [] → SiteA1
        phos_A2 : [] → SiteA2
        !K1 : [] → SiteK1
        !K2 : [] → SiteK2
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [k1] : [SiteK1] ⊢
          phosphorylate_1 [k1]
            : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [i_K1 [k1]]])
            → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [i_K1 [k1]]])
        [k2] : [SiteK2] ⊢
          phosphorylate_2 [k2]
            : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [k2]]])
            → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [k2]]])
    "#]];
    expected.assert_eq(&model().to_string());
}

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [i_A1 [unphos_A1 []]]]
        Agent [i_A [i_A1 [phos_A1 []]]]
        Agent [i_A [i_A2 [unphos_A2 []]]]
        Agent [i_A [i_A2 [phos_A2 []]]]
        Agent [i_K [i_K1 [!K1 []]]]
        Agent [i_K [i_K2 [!K2 []]]]"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        phosphorylate_1 [!K1 []]
          : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [i_K1 [!K1 []]]])
          → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [i_K1 [!K1 []]]])
        phosphorylate_2 [!K2 []]
          : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [!K2 []]]])
          → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [!K2 []]]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// ==========================================
// --- Model update: A1 can be phosphorylated also by any K
//
// TODO: Get this to work for "K2 phosphorylates any A". The first problem here is, that this would require a morphism
// `phos: I -> SiteA` instead of `phos: I -> SiteA1`. But without `phos: I -> SiteA1` you can no longer generate the fine-grained
// reaction. This problem may be similar to
// [**Generalized expression**](https://github.com/ToposInstitute/sys-bio-collab/blob/441abbf2359fbd0fe45c99ad4e5a1ef617ccc393/notes/examples/Incremental%20Examples.md?plain=1#L15):
// there we want to turn Gene_i to Protein_i for all possible i. Here, we want to turn A_i to phosphoA_i for all possible i.
// Once this is fixed,
// - Make sure we don't get the reaction K2 phosphorylates A1 twice (once for "any K phosphorylates A1", once for "K2 phosphorylates any A")
// - try to see if this also works for binding reactions (A1 binds to B, A2 binds to B2). The added complexity
// here is that we have a bond going to the tensor product SiteA1 ⊗ SiteB; but we cannot use this path, because it won't expand
// to SiteB1 and SiteB2 separately.
// https://q.uiver.app/#q=WzAsNCxbMSwwLCJTaXRlQTEgXFxvdGltZXMgU2l0ZUIiXSxbMCwxLCJTaXRlQTEgXFxvdGltZXMgU2l0ZUIxIl0sWzIsMSwiU2l0ZUExIFxcb3RpbWVzIFNpdGVCMiJdLFsxLDIsIkkiXSxbMSwwLCJpZF97QTF9IFxcb3RpbWVzIFxcaW90YV97QjF9IiwxXSxbMiwwLCJpZF97QTF9IFxcb3RpbWVzIFxcaW90YV97QjJ9IiwxXSxbMywxLCJib25kX3sxMX0iLDFdLFszLDIsImJvbmRfezEyfSIsMV0sWzMsMCwiYm9uZCIsMSx7InN0eWxlIjp7ImJvZHkiOnsibmFtZSI6ImRhc2hlZCJ9fX1dXQ==

/// Knowledge update (generalization)
fn model_decl_updated() -> [ModelDecl; 3] {
    use crate::surface::*;
    // Define agent
    let agent = ModelDecl::agent("Agent", [ObTm::var("a")], [Ty::sort("TyAgent")]);

    // Define patterns
    let a1 = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("i_A1", [MorTm::var("s1")])])]);
    let a2 = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("i_A2", [MorTm::var("s2")])])]);
    let k = PatTm::res("Agent", [MorTm::app("i_K", [MorTm::var("k")])]);
    // let k1 =
    // PatTm::res("Agent", [MorTm::app("i_K", [MorTm::app("i_K1", [MorTm::app("!K1", [])])])]);
    let k2 = PatTm::res("Agent", [MorTm::app("i_K", [MorTm::app("i_K2", [MorTm::var("k2")])])]);
    let a1_unphos = a1.subst(&mut vec![(name("s1"), MorTm::app("unphos_A1", []))]);
    let a2_unphos = a2.subst(&mut vec![(name("s2"), MorTm::app("unphos_A2", []))]);
    let a1_phos = a1.subst(&mut vec![(name("s1"), MorTm::app("phos_A1", []))]);
    let a2_phos = a2.subst(&mut vec![(name("s2"), MorTm::app("phos_A2", []))]);
    // Define rules
    let phosphorylate_1 = ModelDecl::rule(
        "phosphorylate_1",
        [ObTm::var("k")],
        [Ty::sort("SiteK")],
        PatTm::tensor([a1_unphos, k.clone()]),
        PatTm::tensor([a1_phos, k]),
    );
    let phosphorylate_2 = ModelDecl::rule(
        "phosphorylate_2",
        [ObTm::var("k2")],
        [Ty::sort("SiteK2")],
        PatTm::tensor([a2_unphos, k2.clone()]),
        PatTm::tensor([a2_phos, k2]),
    );
    [agent, phosphorylate_1, phosphorylate_2]
}

fn model_updated() -> Model {
    let decls = model_decl_updated();
    Model::parse(signature(), decls).unwrap()
}

#[test]
fn parse_model_updated() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteK
        SiteA1
        SiteK1
        SiteA2
        SiteK2
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_K : [SiteK] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_K1 : [SiteK1] → SiteK
        i_K2 : [SiteK2] → SiteK
        unphos_A1 : [] → SiteA1
        unphos_A2 : [] → SiteA2
        phos_A1 : [] → SiteA1
        phos_A2 : [] → SiteA2
        !K1 : [] → SiteK1
        !K2 : [] → SiteK2
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [k] : [SiteK] ⊢
          phosphorylate_1 [k]
            : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [k]])
            → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [k]])
        [k2] : [SiteK2] ⊢
          phosphorylate_2 [k2]
            : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [k2]]])
            → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [k2]]])
    "#]];
    expected.assert_eq(&model_updated().to_string());
}

#[test]
fn generate_network_updated() {
    use itertools::Itertools;
    let model = model_updated();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [i_A1 [unphos_A1 []]]]
        Agent [i_A [i_A1 [phos_A1 []]]]
        Agent [i_A [i_A2 [unphos_A2 []]]]
        Agent [i_A [i_A2 [phos_A2 []]]]
        Agent [i_K [i_K1 [!K1 []]]]
        Agent [i_K [i_K2 [!K2 []]]]"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        phosphorylate_1 [i_K1 [!K1 []]]
          : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [i_K1 [!K1 []]]])
          → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [i_K1 [!K1 []]]])
        phosphorylate_1 [i_K2 [!K2 []]]
          : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [i_K2 [!K2 []]]])
          → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [i_K2 [!K2 []]]])
        phosphorylate_2 [!K2 []]
          : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [!K2 []]]])
          → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [!K2 []]]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// ==========================================
// --- Coarse graining (coproduct)
// ==========================================

// Coarse grained signature
fn grounding_signature_coproduct() -> Signature {
    let d = [
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteA1"),
        SignatureDecl::sort("SiteA2"),
        SignatureDecl::sort("SiteK"),
        // Operations
        SignatureDecl::operation("unphos_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("unphos_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("phos_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("phos_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("!K", [], Ty::sort("SiteK")),
    ];
    Signature::parse(d).unwrap()
}

/// Full signature.
fn signature_coproduct() -> Signature {
    let sig1 = main_signature();
    let sig2 = grounding_signature_coproduct();
    merge_signatures(&[sig1, sig2])
}

fn model_updated_coproduct() -> Model {
    let decls = model_decl_updated();
    Model::parse(signature_coproduct(), decls).unwrap()
}

#[test]
fn parse_model_updated_coproduct() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteK
        SiteA1
        SiteK1
        SiteA2
        SiteK2
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_K : [SiteK] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_K1 : [SiteK1] → SiteK
        i_K2 : [SiteK2] → SiteK
        unphos_A1 : [] → SiteA1
        unphos_A2 : [] → SiteA2
        phos_A1 : [] → SiteA1
        phos_A2 : [] → SiteA2
        !K : [] → SiteK
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [k] : [SiteK] ⊢
          phosphorylate_1 [k]
            : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [k]])
            → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [k]])
        [k2] : [SiteK2] ⊢
          phosphorylate_2 [k2]
            : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [k2]]])
            → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [k2]]])
    "#]];
    expected.assert_eq(&model_updated_coproduct().to_string());
}

#[test]
fn generate_network_updated_coproduct() {
    use itertools::Itertools;
    let model = model_updated_coproduct();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [i_A1 [unphos_A1 []]]]
        Agent [i_A [i_A1 [phos_A1 []]]]
        Agent [i_A [i_A2 [unphos_A2 []]]]
        Agent [i_A [i_A2 [phos_A2 []]]]
        Agent [i_K [!K []]]"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        phosphorylate_1 [!K []]
          : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [!K []]])
          → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [!K []]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// ==========================================
// --- Coarse graining (product)
// ==========================================

/// Coarse grained signature
fn grounding_signature_product() -> Signature {
    let d = [
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteA1"),
        SignatureDecl::sort("SiteA2"),
        SignatureDecl::sort("SiteK1"),
        SignatureDecl::sort("SiteK2"),
        SignatureDecl::sort("SiteK*"),
        // Operations
        SignatureDecl::operation("unphos_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("unphos_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("phos_A1", [], Ty::sort("SiteA1")),
        SignatureDecl::operation("phos_A2", [], Ty::sort("SiteA2")),
        SignatureDecl::operation("i_K*1", [Ty::sort("SiteK*")], Ty::sort("SiteK1")),
        SignatureDecl::operation("i_K*2", [Ty::sort("SiteK*")], Ty::sort("SiteK2")),
        SignatureDecl::operation("!K*", [], Ty::sort("SiteK*")),
    ];
    Signature::parse(d).unwrap()
}

/// Full signature.
fn signature_product() -> Signature {
    let sig1 = main_signature();
    let sig2 = grounding_signature_product();
    merge_signatures(&[sig1, sig2])
}

fn model_product() -> Model {
    let decls = model_decl();
    Model::parse(signature_product(), decls).unwrap()
}

#[test]
fn parse_model_product() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        SiteA
        SiteK
        SiteA1
        SiteK1
        SiteA2
        SiteK2
        SiteK*
        #/ operations:
        i_A : [SiteA] → TyAgent
        i_K : [SiteK] → TyAgent
        i_A1 : [SiteA1] → SiteA
        i_A2 : [SiteA2] → SiteA
        i_K1 : [SiteK1] → SiteK
        i_K2 : [SiteK2] → SiteK
        unphos_A1 : [] → SiteA1
        unphos_A2 : [] → SiteA2
        phos_A1 : [] → SiteA1
        phos_A2 : [] → SiteA2
        i_K*1 : [SiteK*] → SiteK1
        i_K*2 : [SiteK*] → SiteK2
        !K* : [] → SiteK*
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [k1] : [SiteK1] ⊢
          phosphorylate_1 [k1]
            : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [i_K1 [k1]]])
            → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [i_K1 [k1]]])
        [k2] : [SiteK2] ⊢
          phosphorylate_2 [k2]
            : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [k2]]])
            → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [k2]]])
    "#]];
    expected.assert_eq(&model_product().to_string());
}

#[test]
fn generate_network_product() {
    use itertools::Itertools;
    let model = model_product();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [i_A1 [unphos_A1 []]]]
        Agent [i_A [i_A1 [phos_A1 []]]]
        Agent [i_A [i_A2 [unphos_A2 []]]]
        Agent [i_A [i_A2 [phos_A2 []]]]
        Agent [i_K [i_K1 [i_K*1 [!K* []]]]]
        Agent [i_K [i_K2 [i_K*2 [!K* []]]]]"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        phosphorylate_1 [i_K*1 [!K* []]]
          : (Agent [i_A [i_A1 [unphos_A1 []]]], Agent [i_K [i_K1 [i_K*1 [!K* []]]]])
          → (Agent [i_A [i_A1 [phos_A1 []]]], Agent [i_K [i_K1 [i_K*1 [!K* []]]]])
        phosphorylate_2 [i_K*2 [!K* []]]
          : (Agent [i_A [i_A2 [unphos_A2 []]]], Agent [i_K [i_K2 [i_K*2 [!K* []]]]])
          → (Agent [i_A [i_A2 [phos_A2 []]]], Agent [i_K [i_K2 [i_K*2 [!K* []]]]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
