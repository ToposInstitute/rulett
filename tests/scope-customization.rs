//! https://q.uiver.app/#q=WzAsNDIsWzEsMSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsMSwiU2l0ZUIiXSxbMiwwLCJUeUFnZW50Il0sWzYsMCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSwyLCJJIl0sWzMsMiwiSSJdLFs0LDAsIlNpdGVBIl0sWzUsMCwiU2l0ZUIiXSxbNywwLCJSZXMiXSxbNywyLCJJIl0sWzEsNSwiUmVzIFxcb3RpbWVzIFNpdGVBIl0sWzIsNSwiU2l0ZUIiXSxbMiw0LCJUeUFnZW50Il0sWzYsNCwiU2l0ZUEgXFxvdGltZXMgU2l0ZUIiXSxbNSw1LCJTX2IiXSxbMyw1LCJTX2siXSxbNCw0LCJTaXRlQSJdLFs1LDQsIlNpdGVCIl0sWzcsNCwiUmVzIl0sWzcsNSwiU19wIl0sWzMsNiwiSSJdLFs1LDYsIkkiXSxbNyw2LCJJIl0sWzEsOSwiU2l0ZUEiXSxbMiw5LCJTaXRlQiJdLFsyLDgsIlR5QWdlbnQiXSxbOCw4LCJTaXRlQTEgXFxvdGltZXMgU2l0ZUIxIl0sWzYsMTAsIlNfYiJdLFs1LDgsIlNpdGVBIl0sWzYsOCwiU2l0ZUIiXSxbNiwxMSwiSSJdLFs0LDksIlNpdGVBMSJdLFs1LDksIlNpdGVBMiJdLFs2LDksIlNpdGVCMSJdLFs3LDksIlNpdGVCMiJdLFs5LDgsIlNpdGVBMiBcXG90aW1lcyBTaXRlQjIiXSxbOCwxMCwiU197YjF9Il0sWzksMTAsIlNfe2IyfSJdLFswLDEwLCJTaXRlQTEiXSxbMSwxMCwiU2l0ZUEyIl0sWzIsMTAsIlNpdGVCMSJdLFszLDEwLCJTaXRlQjIiXSxbNSwyLCJcXGlvdGFfSyIsMl0sWzQsNywiZW1wdHlfQiIsMV0sWzksOCwicGhvcyIsMCx7ImN1cnZlIjotMX1dLFs0LDYsImVtcHR5X0EiXSxbNCwzLCJib25kX3tBQn0iLDJdLFswLDIsIlxcaW90YV9BIiwyXSxbMSwyLCJcXGlvdGFfQiIsMl0sWzE1LDEyXSxbMTQsMTcsImVtcHR5X0IiLDFdLFsxOSwxOCwicCIsMCx7ImN1cnZlIjotMX1dLFsxNCwxNiwiZW1wdHlfQSJdLFsxNCwxMywiYm9uZF97QUJ9IiwyXSxbMTAsMTJdLFsxMSwxMl0sWzIwLDE1XSxbMjEsMTRdLFsyMiwxOV0sWzksOCwidW5waG9zIiwyLHsiY3VydmUiOjF9XSxbMTksMTgsInUiLDIseyJjdXJ2ZSI6MX1dLFsyMywyNV0sWzI0LDI1XSxbMzAsMjddLFszMSwyOF0sWzI3LDMxLCJlbXB0eV9BIl0sWzMyLDI4XSxbMjcsMzJdLFszNCwyOV0sWzI3LDMzLCJlbXB0eV9CIiwxXSxbMzMsMjldLFsyNywzNF0sWzM2LDI2XSxbMzcsMzVdLFszOCwyM10sWzM5LDIzXSxbNDAsMjRdLFs0MSwyNF1d

mod common;
use common::*;
use rulett::prelude::name;

/// Base signature.
fn main_signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyAgent"),
        SignatureDecl::sort("Res"),
        SignatureDecl::sort("SiteA"),
        SignatureDecl::sort("SiteB"),
        // Separation layer to `[]`
        SignatureDecl::sort("S_k"),
        SignatureDecl::sort("S_b"),
        SignatureDecl::sort("S_p"),
        // Operations
        SignatureDecl::operation("i_A", [Ty::sort("Res"), Ty::sort("SiteA")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_B", [Ty::sort("SiteB")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_K", [Ty::sort("S_k")], Ty::sort("TyAgent")),
        SignatureDecl::operation("empty_A", [Ty::sort("S_b")], Ty::sort("SiteA")),
        SignatureDecl::operation("empty_B", [Ty::sort("S_b")], Ty::sort("SiteB")),
        SignatureDecl::operation(
            "bond_AB",
            [Ty::sort("S_b")],
            Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")]),
        ),
        SignatureDecl::operation("phos", [Ty::sort("S_p")], Ty::sort("Res")),
        SignatureDecl::operation("unphos", [Ty::sort("S_p")], Ty::sort("Res")),
    ])
    .unwrap()
}

/// Grounding signature.
fn grounding_signature(skip_kinase: bool) -> Signature {
    let d1 = [
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("S_k"),
        SignatureDecl::sort("S_b"),
        SignatureDecl::sort("S_p"),
        // Operations
        SignatureDecl::operation("!k", [], Ty::sort("S_k")),
        SignatureDecl::operation("!b", [], Ty::sort("S_b")),
        SignatureDecl::operation("!p", [], Ty::sort("S_p")),
    ];
    let d2 = [
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("S_b"),
        SignatureDecl::sort("S_p"),
        // Operations
        SignatureDecl::operation("!b", [], Ty::sort("S_b")),
        SignatureDecl::operation("!p", [], Ty::sort("S_p")),
    ];
    if skip_kinase {
        Signature::parse(d2).unwrap()
    } else {
        Signature::parse(d1).unwrap()
    }
}

/// Full signature.
fn signature(skip_kinase: bool) -> Signature {
    let sig1 = main_signature();
    println!("Skip Kinase: {skip_kinase}");
    let sig2 = grounding_signature(skip_kinase);
    merge_signatures(&[sig1, sig2])
}

// Declares Model.
fn model_decl() -> [ModelDecl; 3] {
    use crate::surface::*;
    // Define agent
    let agent = ModelDecl::agent("Agent", [ObTm::var("a")], [Ty::sort("TyAgent")]);

    // Define patterns
    let a_free = PatTm::res(
        "Agent",
        [MorTm::app(
            "i_A",
            [MorTm::tensor([MorTm::var("r"), MorTm::app("empty_A", [MorTm::var("b")])])],
        )],
    );
    let b_free =
        PatTm::res("Agent", [MorTm::app("i_B", [MorTm::app("empty_B", [MorTm::var("b")])])]);
    let a_s1 = PatTm::res(
        "Agent",
        [MorTm::app("i_A", [MorTm::tensor([MorTm::var("r"), MorTm::var("s1")])])],
    );
    let b_s2 = PatTm::res("Agent", [MorTm::app("i_B", [MorTm::var("s2")])]);
    let ab = PatTm::let_(
        ObTm::tensor([ObTm::var("s1"), ObTm::var("s2")]),
        MorTm::app("bond_AB", [MorTm::var("b")]),
        PatTm::tensor([a_s1, b_s2]),
    );
    let a_unphos =
        PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("unphos", [MorTm::var("p")])])]);
    let a_phos = a_unphos.subst(&mut vec![(name("unphos"), MorTm::var("phos"))]);
    let k = PatTm::res("Agent", [MorTm::app("i_K", [MorTm::var("k")])]);

    // Define rules
    let bond_ab = ModelDecl::rule(
        "bondAB",
        [ObTm::var("r"), ObTm::var("b")],
        [Ty::sort("Res"), Ty::sort("S_b")],
        PatTm::tensor([a_free, b_free]),
        ab,
    ); // @Evan: here, we use the variable `b` twice on the lhs and twice on the rhs
    let phosphorylate = ModelDecl::rule(
        "phosphorylate",
        [ObTm::var("s"), ObTm::var("k")],
        [Ty::sort("SiteA"), Ty::sort("S_k")],
        PatTm::tensor([a_unphos, k.clone()]),
        PatTm::tensor([a_phos, k]),
    ); // @Evan" here we are using the variable `k` once on the lhs and once on the rhs.
    [agent, bond_ab, phosphorylate]
}

// Generates Model.
fn model(skip_kinase: bool) -> Model {
    let decls = model_decl();
    Model::parse(signature(skip_kinase), decls).unwrap()
}

// With kinase
#[test]
fn parse_signature() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_p
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_b] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [S_b] → ⊗ [SiteA, SiteB]
        phos : [S_p] → Res
        unphos : [S_p] → Res
        !k : [] → S_k
        !b : [] → S_b
        !p : [] → S_p
    "#]];
    expected.assert_eq(&signature(false).to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_p
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_b] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [S_b] → ⊗ [SiteA, SiteB]
        phos : [S_p] → Res
        unphos : [S_p] → Res
        !k : [] → S_k
        !b : [] → S_b
        !p : [] → S_p
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r, b] : [Res, S_b] ⊢
          bondAB [r, b]
            : (Agent [i_A [(r, empty_A [b])]], Agent [i_B [empty_B [b]]])
            → let bond_AB [b] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s, k] : [SiteA, S_k] ⊢
          phosphorylate [s, k]
            : (Agent [i_A [unphos [p]]], Agent [i_K [k]])
            → (Agent [i_A [unphos [p]]], Agent [i_K [k]])
    "#]];
    expected.assert_eq(&model(false).to_string());
}

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model(false);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [phos [!p []], empty_A [!b []]]]
        Agent [i_A [unphos [!p []], empty_A [!b []]]]
        Agent [i_B [empty_B [!b []]]]
        Agent [i_K [!k []]]
        let bond_AB [!b []] in (Agent [i_A [phos [!p []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!b []] in (Agent [i_A [unphos [!p []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!b []] in (Agent [i_B [0.1]], Agent [i_A [phos [!p []], 0.0]])
        let bond_AB [!b []] in (Agent [i_B [0.1]], Agent [i_A [unphos [!p []], 0.0]])"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bondAB [phos [!p []], !b []]
          : (
            Agent [i_A [(phos [!p []], empty_A [!b []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [!b []] in
            (Agent [i_A [(phos [!p []], 0.0)]], Agent [i_B [0.1]])
        bondAB [unphos [!p []], !b []]
          : (
            Agent [i_A [(unphos [!p []], empty_A [!b []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [!b []] in
            (Agent [i_A [(unphos [!p []], 0.0)]], Agent [i_B [0.1]])
        phosphorylate [empty_A [!b []], !k []]
          : (Agent [i_A [unphos [p]]], Agent [i_K [!k []]])
          → (Agent [i_A [unphos [p]]], Agent [i_K [!k []]])
        let bond_AB [!b []] in (Agent [i_B [0.1]], phosphorylate [0.0, !k []])
          : let bond_AB [!b []] in
            (Agent [i_B [0.1]], (Agent [i_A [unphos [p]]], Agent [i_K [!k []]]))
          → let bond_AB [!b []] in
            (Agent [i_B [0.1]], (Agent [i_A [unphos [p]]], Agent [i_K [!k []]]))"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// Without kinase
#[test]
fn parse_signature_no_kinase() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_p
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_b] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [S_b] → ⊗ [SiteA, SiteB]
        phos : [S_p] → Res
        unphos : [S_p] → Res
        !b : [] → S_b
        !p : [] → S_p
    "#]];
    expected.assert_eq(&signature(true).to_string());
}

#[test]
fn parse_model_no_kinase() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_p
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_b] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [S_b] → ⊗ [SiteA, SiteB]
        phos : [S_p] → Res
        unphos : [S_p] → Res
        !b : [] → S_b
        !p : [] → S_p
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r, b] : [Res, S_b] ⊢
          bondAB [r, b]
            : (Agent [i_A [(r, empty_A [b])]], Agent [i_B [empty_B [b]]])
            → let bond_AB [b] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s, k] : [SiteA, S_k] ⊢
          phosphorylate [s, k]
            : (Agent [i_A [unphos [p]]], Agent [i_K [k]])
            → (Agent [i_A [unphos [p]]], Agent [i_K [k]])
    "#]];
    expected.assert_eq(&model(true).to_string());
}

#[test]
fn generate_network_no_kinase() {
    use itertools::Itertools;
    let model = model(true);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [phos [!p []], empty_A [!b []]]]
        Agent [i_A [unphos [!p []], empty_A [!b []]]]
        Agent [i_B [empty_B [!b []]]]
        let bond_AB [!b []] in (Agent [i_A [phos [!p []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!b []] in (Agent [i_A [unphos [!p []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!b []] in (Agent [i_B [0.1]], Agent [i_A [phos [!p []], 0.0]])
        let bond_AB [!b []] in (Agent [i_B [0.1]], Agent [i_A [unphos [!p []], 0.0]])"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bondAB [phos [!p []], !b []]
          : (
            Agent [i_A [(phos [!p []], empty_A [!b []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [!b []] in
            (Agent [i_A [(phos [!p []], 0.0)]], Agent [i_B [0.1]])
        bondAB [unphos [!p []], !b []]
          : (
            Agent [i_A [(unphos [!p []], empty_A [!b []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [!b []] in
            (Agent [i_A [(unphos [!p []], 0.0)]], Agent [i_B [0.1]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
