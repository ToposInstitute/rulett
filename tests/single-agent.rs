//! https://q.uiver.app/#q=WzAsNDIsWzEsMSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsMSwiU2l0ZUIiXSxbMiwwLCJUeUFnZW50Il0sWzYsMCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSwyLCJJIl0sWzMsMiwiSSJdLFs0LDAsIlNpdGVBIl0sWzUsMCwiU2l0ZUIiXSxbNywwLCJSZXMiXSxbNywyLCJJIl0sWzEsNSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsNSwiU2l0ZUIiXSxbMiw0LCJUeUFnZW50Il0sWzYsNCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSw1LCJTX2IiXSxbMyw1LCJTaXRlMCJdLFs0LDQsIlNpdGVBIl0sWzUsNCwiU2l0ZUIiXSxbNyw0LCJSZXMiXSxbNyw1LCJTX3AiXSxbMyw2LCJJIl0sWzUsNiwiSSJdLFs3LDYsIkkiXSxbMSw5LCJTaXRlQSJdLFsyLDksIlNpdGVCIl0sWzIsOCwiVHlBZ2VudCJdLFs4LDgsIlNpdGVBMSBcXG90aW1lcyBTaXRlQjEiXSxbNiwxMCwiU19iIl0sWzUsOCwiU2l0ZUEiXSxbNiw4LCJTaXRlQiJdLFs2LDExLCJJIl0sWzQsOSwiU2l0ZUExIl0sWzUsOSwiU2l0ZUEyIl0sWzYsOSwiU2l0ZUIxIl0sWzcsOSwiU2l0ZUIyIl0sWzksOCwiU2l0ZUEyIFxcb3RpbWVzIFNpdGVCMiJdLFs4LDEwLCJTX3tiMX0iXSxbOSwxMCwiU197YjJ9Il0sWzAsMTAsIlNpdGVBMSJdLFsxLDEwLCJTaXRlQTIiXSxbMiwxMCwiU2l0ZUIxIl0sWzMsMTAsIlNpdGVCMiJdLFs1LDIsIlxcaW90YV9LIiwyXSxbNCw3LCJlbXB0eV9CIiwxXSxbOSw4LCJwaG9zIiwwLHsiY3VydmUiOi0xfV0sWzQsNiwiZW1wdHlfQSJdLFs0LDMsImJvbmRfe0FCfSIsMl0sWzAsMiwiXFxpb3RhX0EiLDJdLFsxLDIsIlxcaW90YV9CIiwyXSxbMTUsMTJdLFsxNCwxNywiZW1wdHlfQiIsMV0sWzE5LDE4LCJwIiwwLHsiY3VydmUiOi0xfV0sWzE0LDE2LCJlbXB0eV9BIl0sWzE0LDEzLCJib25kX3tBQn0iLDJdLFsxMCwxMl0sWzExLDEyXSxbMjAsMTVdLFsyMSwxNF0sWzIyLDE5XSxbOSw4LCJ1bnBob3MiLDIseyJjdXJ2ZSI6MX1dLFsxOSwxOCwidSIsMix7ImN1cnZlIjoxfV0sWzIzLDI1XSxbMjQsMjVdLFszMCwyN10sWzMxLDI4XSxbMjcsMzEsImVtcHR5X0EiXSxbMzIsMjhdLFsyNywzMl0sWzM0LDI5XSxbMjcsMzMsImVtcHR5X0IiLDFdLFszMywyOV0sWzI3LDM0XSxbMzYsMjZdLFszNywzNV0sWzM4LDIzXSxbMzksMjNdLFs0MCwyNF0sWzQxLDI0XV0=

mod common;
use common::*;
use rulett::prelude::name;

/// Base signature.
fn signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyAgent"),
        SignatureDecl::sort("Res"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        // Operations
        SignatureDecl::operation("i_A", [Ty::sort("Res"), Ty::sort("SiteA")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_B", [Ty::sort("SiteB")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_K", [], Ty::sort("TyAgent")),
        SignatureDecl::operation("empty_A", [], Ty::sort("SiteA")),
        SignatureDecl::operation("empty_B", [], Ty::sort("SiteB")),
        SignatureDecl::operation("bond_AB", [], Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")])),
        SignatureDecl::operation("phos", [], Ty::sort("Res")),
        SignatureDecl::operation("unphos", [], Ty::sort("Res")),
    ])
    .unwrap()
}

// Declares Model.
fn model_decl() -> [ModelDecl; 3] {
    use crate::surface::*;
    // Define agent
    let agent = ModelDecl::agent("Agent", [ObTm::var("a")], [Ty::sort("TyAgent")]);

    // Define patterns
    let a_free = PatTm::res(
        "Agent",
        [MorTm::app("i_A", MorTm::tensor([MorTm::var("r"), MorTm::app("empty_A", [])]))],
    );
    let b_free = PatTm::res("Agent", [MorTm::app("i_B", [MorTm::app("empty_B", [])])]);
    let a_s1 = PatTm::res(
        "Agent",
        [MorTm::app("i_A", [MorTm::tensor([MorTm::var("r"), MorTm::var("s1")])])],
    );
    let b_s2 = PatTm::res("Agent", [MorTm::app("i_B", [MorTm::var("s2")])]);
    let ab = PatTm::let_(
        ObTm::tensor([ObTm::var("s1"), ObTm::var("s2")]),
        MorTm::app("bond", []),
        PatTm::tensor([a_s1, b_s2]),
    );
    let a_unphos = PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("unphos", [])])]);
    let a_phos = a_unphos.subst(&mut vec![(name("unphos"), MorTm::var("phos"))]);
    let k = PatTm::res("Agent", [MorTm::app("i_K", [])]);

    // Define rules
    let bond_ab = ModelDecl::rule(
        "bondAB",
        [ObTm::var("r")],
        [Ty::sort("Res")],
        PatTm::tensor([a_free, b_free]),
        ab,
    );
    let phosphorylate = ModelDecl::rule(
        "phosphorylate",
        [ObTm::var("s")],
        [Ty::sort("SiteA")],
        PatTm::tensor([a_unphos, k.clone()]),
        PatTm::tensor([a_phos, k]),
    );
    [agent, bond_ab, phosphorylate]
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
        Res
        SiteA
        SiteB
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [] → TyAgent
        empty_A : [] → SiteA
        empty_B : [] → SiteB
        bond_AB : [] → ⊗ [SiteA, SiteB]
        phos : [] → Res
        unphos : [] → Res
    "#]];
    expected.assert_eq(&signature().to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [] → TyAgent
        empty_A : [] → SiteA
        empty_B : [] → SiteB
        bond_AB : [] → ⊗ [SiteA, SiteB]
        phos : [] → Res
        unphos : [] → Res
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r] : [Res] ⊢
          bondAB [r]
            : (Agent [i_A (r, empty_A [])], Agent [i_B [empty_B []]])
            → let bond [] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s] : [SiteA] ⊢
          phosphorylate [s]
            : (Agent [i_A [unphos []]], Agent [i_K []])
            → (Agent [i_A [unphos []]], Agent [i_K []])
    "#]];
    expected.assert_eq(&model().to_string());
}

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [phos [], empty_A []]]
        Agent [i_A [unphos [], empty_A []]]
        Agent [i_B [empty_B []]]
        Agent [i_K []]
        let bond_AB [] in (Agent [i_A [phos [], 0.0]], Agent [i_B [0.1]])
        let bond_AB [] in (Agent [i_A [unphos [], 0.0]], Agent [i_B [0.1]])
        let bond_AB [] in (Agent [i_B [0.1]], Agent [i_A [phos [], 0.0]])
        let bond_AB [] in (Agent [i_B [0.1]], Agent [i_A [unphos [], 0.0]])"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bondAB [phos []]
          : (Agent [i_A (phos [], empty_A [])], Agent [i_B [empty_B []]])
          → let bond [] in (Agent [i_A [(phos [], 0.0)]], Agent [i_B [0.1]])
        bondAB [unphos []]
          : (Agent [i_A (unphos [], empty_A [])], Agent [i_B [empty_B []]])
          → let bond [] in (Agent [i_A [(unphos [], 0.0)]], Agent [i_B [0.1]])
        phosphorylate [empty_A []]
          : (Agent [i_A [unphos []]], Agent [i_K []])
          → (Agent [i_A [unphos []]], Agent [i_K []])
        let bond_AB [] in (Agent [i_B [0.1]], phosphorylate [0.0])
          : let bond_AB [] in
            (Agent [i_B [0.1]], (Agent [i_A [unphos []]], Agent [i_K []]))
          → let bond_AB [] in
            (Agent [i_B [0.1]], (Agent [i_A [unphos []]], Agent [i_K []]))"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
