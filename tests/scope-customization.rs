//! This example shows how we can customize the scope of our vanilla association-phosphorylation model.
//! This is done by introducing a separation layer between our previous sites and []. In particular,
//! we choose one separation sort for each of the agents A, B and K. Each site on agent X is connected
//! to separation sort S_x through a morphism descriping the state of the site. For example: `phos: S_a -> Res`.
//! Users can then select which agents X should be part of the generated reaction network by defining
//! a unique morphism !x: [] -> S_x. Note that this approach assumes that sites are not shared across agents.
//!
//! https://q.uiver.app/#q=WzAsMTQsWzEsMCwiXFxtYXRocm17VHlBZ2VudH0iXSxbMCwxLCJcXG1hdGhybXtTaXRlQX0gXFxvdGltZXMgXFxtYXRocm17UmVzfSJdLFsxLDEsIlxcbWF0aHJte1NpdGVCfSJdLFsyLDEsIlNfayJdLFszLDAsIlxcbWF0aHJte1NpdGVBfSJdLFs0LDAsIlxcbWF0aHJte1NpdGVCfSJdLFs1LDAsIlxcbWF0aHJte1NpdGVBfSBcXG90aW1lcyBcXG1hdGhybXtTaXRlQn0iXSxbNiwwLCJcXG1hdGhybXtSZXN9Il0sWzMsMSwiU19hIl0sWzYsMSwiU19hIl0sWzAsMCwiXFxtYXRoc2Z7VH0iXSxbNCwxLCJTX2IiXSxbNSwxLCJTX2EgXFxvdGltZXMgU19iIl0sWzMsMiwiSSJdLFsxLDAsIlxcaW90YV9BIl0sWzIsMCwiXFxpb3RhX0IiLDJdLFszLDAsIlxcaW90YV9LIiwyXSxbOSw3LCJcXG1hdGhybXtwaG9zfSIsMCx7ImN1cnZlIjotMX1dLFs5LDcsIlxcbWF0aHJte3VucGhvc30iLDIseyJjdXJ2ZSI6MX1dLFs4LDQsIlxcbWF0aHJte2VtcHR5X0F9IiwxXSxbMTEsNSwiXFxtYXRocm17ZW1wdHlfQn0iLDFdLFsxMiw2LCJcXG1hdGhybXtib25kX3tBQn19IiwxXSxbMTMsMywiIWsiXSxbMTMsOCwiIWEiLDFdLFsxMywxMSwiIWIiLDIseyJjb2xvdXIiOlswLDAsNTBdfSxbMCwwLDUwLDFdXV0=

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
        SignatureDecl::sort("S_a"),
        // Operations
        SignatureDecl::operation("i_A", [Ty::sort("Res"), Ty::sort("SiteA")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_B", [Ty::sort("SiteB")], Ty::sort("TyAgent")),
        SignatureDecl::operation("i_K", [Ty::sort("S_k")], Ty::sort("TyAgent")),
        SignatureDecl::operation("empty_A", [Ty::sort("S_a")], Ty::sort("SiteA")),
        SignatureDecl::operation("empty_B", [Ty::sort("S_b")], Ty::sort("SiteB")),
        SignatureDecl::operation(
            "bond_AB",
            [Ty::tensor([Ty::sort("S_a"), Ty::sort("S_b")])],
            Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")]),
        ),
        SignatureDecl::operation("phos", [Ty::sort("S_a")], Ty::sort("Res")),
        SignatureDecl::operation("unphos", [Ty::sort("S_a")], Ty::sort("Res")),
    ])
    .unwrap()
}

enum Remove {
    Nothing,
    A,
    B,
    K,
}

/// Grounding signature.
fn grounding_signature(remove: Remove) -> Signature {
    let mut d = vec![
        // Sorts (Separation layer to `[]`)
        SignatureDecl::sort("S_k"),
        SignatureDecl::sort("S_b"),
        SignatureDecl::sort("S_a"),
        // Operations
        SignatureDecl::operation("!k", [], Ty::sort("S_k")),
        SignatureDecl::operation("!b", [], Ty::sort("S_b")),
        SignatureDecl::operation("!a", [], Ty::sort("S_a")),
    ];
    let (sort_idx, op_idx) = match remove {
        Remove::Nothing => return Signature::parse(d).unwrap(),
        Remove::A => (2, 5),
        Remove::B => (1, 4),
        Remove::K => (0, 3),
    };

    d.remove(op_idx);
    d.remove(sort_idx);

    Signature::parse(d).unwrap()
}

/// Full signature.
fn signature(r: Remove) -> Signature {
    let sig1 = main_signature();
    let sig2 = grounding_signature(r);
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
            [MorTm::tensor([MorTm::var("r"), MorTm::app("empty_A", [MorTm::var("a")])])],
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
        MorTm::app("bond_AB", [MorTm::tensor([MorTm::var("a"), MorTm::var("b")])]),
        PatTm::tensor([a_s1, b_s2]),
    );
    let a_unphos =
        PatTm::res("Agent", [MorTm::app("i_A", [MorTm::app("unphos", [MorTm::var("p")])])]);
    let a_phos = a_unphos.subst(&mut vec![(name("unphos"), MorTm::var("phos"))]);
    let k = PatTm::res("Agent", [MorTm::app("i_K", [MorTm::var("k")])]);

    // Define rules
    let bond_ab = ModelDecl::rule(
        "bondAB",
        [ObTm::var("r"), ObTm::var("a"), ObTm::var("b")],
        [Ty::sort("Res"), Ty::sort("S_a"), Ty::sort("S_b")],
        PatTm::tensor([a_free, b_free]),
        ab,
    ); // @Evan: here, we use the variable `a` once on the lhs and once on the rhs. Ditto for `b`. Linearity means once per pattern, not once per rule, right?
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
fn model(r: Remove) -> Model {
    let decls = model_decl();
    Model::parse(signature(r), decls).unwrap()
}

// Remove nothing
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
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !k : [] → S_k
        !b : [] → S_b
        !a : [] → S_a
    "#]];
    expected.assert_eq(&signature(Remove::Nothing).to_string());
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
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !k : [] → S_k
        !b : [] → S_b
        !a : [] → S_a
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r, a, b] : [Res, S_a, S_b] ⊢
          bondAB [r, a, b]
            : (Agent [i_A [(r, empty_A [a])]], Agent [i_B [empty_B [b]]])
            → let bond_AB [(a, b)] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s, k] : [SiteA, S_k] ⊢
          phosphorylate [s, k]
            : (Agent [i_A [unphos [p]]], Agent [i_K [k]])
            → (Agent [i_A [unphos [p]]], Agent [i_K [k]])
    "#]];
    expected.assert_eq(&model(Remove::Nothing).to_string());
}

#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model(Remove::Nothing);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [phos [!a []], empty_A [!a []]]]
        Agent [i_A [unphos [!a []], empty_A [!a []]]]
        Agent [i_B [empty_B [!b []]]]
        Agent [i_K [!k []]]
        let bond_AB [!a [], !b []] in
          (Agent [i_A [phos [!a []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!a [], !b []] in
          (Agent [i_A [unphos [!a []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!a [], !b []] in
          (Agent [i_B [0.1]], Agent [i_A [phos [!a []], 0.0]])
        let bond_AB [!a [], !b []] in
          (Agent [i_B [0.1]], Agent [i_A [unphos [!a []], 0.0]])"#]];
    species.assert_eq(&generator.species(2).join("\n")); // Symmetry issues

    let transitions = expect![[r#"
        bondAB [phos [!a []], !a [], !b []]
          : (
            Agent [i_A [(phos [!a []], empty_A [!a []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [(!a [], !b [])] in
            (Agent [i_A [(phos [!a []], 0.0)]], Agent [i_B [0.1]])
        bondAB [unphos [!a []], !a [], !b []]
          : (
            Agent [i_A [(unphos [!a []], empty_A [!a []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [(!a [], !b [])] in
            (Agent [i_A [(unphos [!a []], 0.0)]], Agent [i_B [0.1]])
        phosphorylate [empty_A [!a []], !k []]
          : (Agent [i_A [unphos [p]]], Agent [i_K [!k []]])
          → (Agent [i_A [unphos [p]]], Agent [i_K [!k []]])
        let bond_AB [!a [], !b []] in (Agent [i_B [0.1]], phosphorylate [0.0, !k []])
          : let bond_AB [!a [], !b []] in
            (Agent [i_B [0.1]], (Agent [i_A [unphos [p]]], Agent [i_K [!k []]]))
          → let bond_AB [!a [], !b []] in
            (Agent [i_B [0.1]], (Agent [i_A [unphos [p]]], Agent [i_K [!k []]]))"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// Without A
#[test]
fn parse_signature_no_a() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !k : [] → S_k
        !b : [] → S_b
    "#]];
    expected.assert_eq(&signature(Remove::A).to_string());
}

#[test]
fn parse_model_no_a() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !k : [] → S_k
        !b : [] → S_b
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r, a, b] : [Res, S_a, S_b] ⊢
          bondAB [r, a, b]
            : (Agent [i_A [(r, empty_A [a])]], Agent [i_B [empty_B [b]]])
            → let bond_AB [(a, b)] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s, k] : [SiteA, S_k] ⊢
          phosphorylate [s, k]
            : (Agent [i_A [unphos [p]]], Agent [i_K [k]])
            → (Agent [i_A [unphos [p]]], Agent [i_K [k]])
    "#]];
    expected.assert_eq(&model(Remove::A).to_string());
}

#[test]
fn generate_network_no_a() {
    use itertools::Itertools;
    let model = model(Remove::A);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_B [empty_B [!b []]]]
        Agent [i_K [!k []]]"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![""];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// Without B
#[test]
fn parse_signature_no_b() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !k : [] → S_k
        !a : [] → S_a
    "#]];
    expected.assert_eq(&signature(Remove::B).to_string());
}

#[test]
fn parse_model_no_b() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !k : [] → S_k
        !a : [] → S_a
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r, a, b] : [Res, S_a, S_b] ⊢
          bondAB [r, a, b]
            : (Agent [i_A [(r, empty_A [a])]], Agent [i_B [empty_B [b]]])
            → let bond_AB [(a, b)] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s, k] : [SiteA, S_k] ⊢
          phosphorylate [s, k]
            : (Agent [i_A [unphos [p]]], Agent [i_K [k]])
            → (Agent [i_A [unphos [p]]], Agent [i_K [k]])
    "#]];
    expected.assert_eq(&model(Remove::B).to_string());
}

#[test]
fn generate_network_no_b() {
    use itertools::Itertools;
    let model = model(Remove::B);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [phos [!a []], empty_A [!a []]]]
        Agent [i_A [unphos [!a []], empty_A [!a []]]]
        Agent [i_K [!k []]]"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![[r#"
        phosphorylate [empty_A [!a []], !k []]
          : (Agent [i_A [unphos [p]]], Agent [i_K [!k []]])
          → (Agent [i_A [unphos [p]]], Agent [i_K [!k []]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// Without A
#[test]
fn parse_signature_no_k() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !b : [] → S_b
        !a : [] → S_a
    "#]];
    expected.assert_eq(&signature(Remove::K).to_string());
}

#[test]
fn parse_model_no_k() {
    let expected = expect![[r#"
        #/ sorts:
        TyAgent
        Res
        SiteA
        SiteB
        S_k
        S_b
        S_a
        #/ operations:
        i_A : [Res, SiteA] → TyAgent
        i_B : [SiteB] → TyAgent
        i_K : [S_k] → TyAgent
        empty_A : [S_a] → SiteA
        empty_B : [S_b] → SiteB
        bond_AB : [⊗ [S_a, S_b]] → ⊗ [SiteA, SiteB]
        phos : [S_a] → Res
        unphos : [S_a] → Res
        !b : [] → S_b
        !a : [] → S_a
        #/ agents:
        [a] : [TyAgent] ⊢ Agent [a]
        #/ rules:
        [r, a, b] : [Res, S_a, S_b] ⊢
          bondAB [r, a, b]
            : (Agent [i_A [(r, empty_A [a])]], Agent [i_B [empty_B [b]]])
            → let bond_AB [(a, b)] in (Agent [i_A [(r, 0.0)]], Agent [i_B [0.1]])
        [s, k] : [SiteA, S_k] ⊢
          phosphorylate [s, k]
            : (Agent [i_A [unphos [p]]], Agent [i_K [k]])
            → (Agent [i_A [unphos [p]]], Agent [i_K [k]])
    "#]];
    expected.assert_eq(&model(Remove::K).to_string());
}

#[test]
fn generate_network_no_k() {
    use itertools::Itertools;
    let model = model(Remove::K);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        Agent [i_A [phos [!a []], empty_A [!a []]]]
        Agent [i_A [unphos [!a []], empty_A [!a []]]]
        Agent [i_B [empty_B [!b []]]]
        let bond_AB [!a [], !b []] in
          (Agent [i_A [phos [!a []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!a [], !b []] in
          (Agent [i_A [unphos [!a []], 0.0]], Agent [i_B [0.1]])
        let bond_AB [!a [], !b []] in
          (Agent [i_B [0.1]], Agent [i_A [phos [!a []], 0.0]])
        let bond_AB [!a [], !b []] in
          (Agent [i_B [0.1]], Agent [i_A [unphos [!a []], 0.0]])"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![[r#"
        bondAB [phos [!a []], !a [], !b []]
          : (
            Agent [i_A [(phos [!a []], empty_A [!a []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [(!a [], !b [])] in
            (Agent [i_A [(phos [!a []], 0.0)]], Agent [i_B [0.1]])
        bondAB [unphos [!a []], !a [], !b []]
          : (
            Agent [i_A [(unphos [!a []], empty_A [!a []])]],
            Agent [i_B [empty_B [!b []]]]
          )
          → let bond_AB [(!a [], !b [])] in
            (Agent [i_A [(unphos [!a []], 0.0)]], Agent [i_B [0.1]])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
