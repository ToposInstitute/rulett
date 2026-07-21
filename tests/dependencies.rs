//! Certain rules have implicit dependencies. For example, a phosphorylation
//! rule can only fire if ATP is present, gene expression can only fire if
//! transcription-translation machinery (RNA-polymerase, ribosomes, etc.)
//! are present, etc.
//!
//! We call the totalty of dependencies an "environment". The required environment for the rule can be
//! made explicit by putting it on either side of a rule. We can use product species in the signature to
//! define environments as shown
//! [here](https://q.uiver.app/#q=WzAsOCxbMSwwLCJUeU1vbGVjdWxlIl0sWzAsMSwiVHlBIl0sWzEsMSwiVHlCIl0sWzIsMSwiVHlDIl0sWzAsMiwiVHlBQiJdLFsxLDIsIlR5QUMiXSxbMiwyLCJUeUJDIl0sWzEsMywiVHlBQkMiXSxbMSwwXSxbMiwwXSxbMywwXSxbNCwxXSxbNSwxXSxbNCwyXSxbNiwyXSxbNSwzXSxbNiwzXSxbNyw0XSxbNyw1XSxbNyw2XV0=)
//! or, using more suggestive labelling,
//! [here](https://q.uiver.app/#q=WzAsOCxbMSwwLCJcXG1hdGhybXtUeU1vbGVjdWxlfSJdLFswLDEsIlxcbWF0aHJte1R5QX0iXSxbMSwxLCJcXG1hdGhybXtUeUJ9Il0sWzIsMSwiXFxtYXRocm17VHlDfSJdLFswLDIsIlxcbWF0aHJte0NlbGxYfSJdLFsxLDIsIlxcbWF0aHJte0NlbGxZfSJdLFsyLDIsIlxcbWF0aHJte0NlbGxafSJdLFsxLDMsIlxcbWF0aHJte0NlbGxPbW5pcG90ZW50fSJdLFsxLDBdLFsyLDBdLFszLDBdLFs0LDFdLFs1LDFdLFs0LDJdLFs2LDJdLFs1LDNdLFs2LDNdLFs3LDRdLFs3LDVdLFs3LDZdXQ==)
//!
//! I.e. an environment is a capability that combines the capabilities of its underlying species.
//!
//! Environments are "made available" in the same way as species: by providing
//! "grounding signatures", that is, morphisms out of `[]`.
//!
//! Note: this requires product species (issue #9), which is currently not a priority.
//! The example below illustrates the phoshporylation of CDC25, which requires either a
//! CDK1.CCNB1 or CDK1.CCNB2 complex. We show that this reaction fires in a cell that expresses only
//! - CDK1 and CCNB1
//! - CDK1 and CCNB2
//!
//! but does not fire in a cell that expresses only
//! - CCNB1 and CCNB2
//!
//! The signature can be illustrated as follows:
//! https://q.uiver.app/#q=WzAsMTQsWzEsMCwiXFxtYXRocm17VHlNb2xlY3VsZX0iXSxbMCwxLCJcXG1hdGhybXtUeUNESzF9Il0sWzEsMSwiXFxtYXRocm17VHlDQ05CMX0iXSxbMiwxLCJcXG1hdGhybXtUeUNDTkIyfSJdLFswLDIsIlxcdGV4dHtDZWxsLWxpbmV9XzEiXSxbMSwyLCJcXHRleHR7Q2VsbC1saW5lfV8yIl0sWzIsMiwiXFx0ZXh0e0NlbGwtbGluZX1fMyJdLFsxLDMsIlxcdGV4dHtDZWxsfV9cXHRleHR7b21uaXBvdGVudH0iXSxbNCwyLCJcXHRleHR7Q2VsbC1saW5lfV8xIl0sWzUsMiwiXFx0ZXh0e0NlbGwtbGluZX1fMiJdLFs2LDIsIlxcdGV4dHtDZWxsLWxpbmV9XzMiXSxbNSwzLCJcXHRleHR7Q2VsbH1fXFx0ZXh0e29tbmlwb3RlbnR9Il0sWzUsMCwiXFxtYXRocm17VHlFbnZ9Il0sWzQsMSwiXFx0ZXh0e0NlbGwtbGluZX1fezEyfSJdLFsxLDBdLFs0LDFdLFs0LDJdLFs1LDFdLFs1LDNdLFs2LDJdLFs2LDNdLFs3LDRdLFs3LDVdLFs3LDZdLFszLDBdLFsyLDBdLFsxMSw4XSxbMTEsOV0sWzEwLDEyXSxbMTEsMTBdLFsxMywxMl0sWzgsMTNdLFs5LDEzXV0=
//!
//! Note that we do not model complex formation explicitly. The purpose for providing an environment is
//! to allow modelers to optionally abstract away such detailed reactions.

mod common;
use common::*;

/// Base signature.
fn base_signature() -> Signature {
    Signature::parse([
        SignatureDecl::sort("TyMolecule"),
        SignatureDecl::sort("TyCDK1"),
        SignatureDecl::sort("TyCCNB1"),
        SignatureDecl::sort("TyCCNB2"),
        SignatureDecl::sort("Res"),
        SignatureDecl::operation("iota_cdk1", [Ty::sort("TyCDK1")], Ty::sort("TyMolecule")),
        SignatureDecl::operation("iota_ccnb1", [Ty::sort("TyCCNB1")], Ty::sort("TyMolecule")),
        SignatureDecl::operation("iota_ccnb2", [Ty::sort("TyCCNB2")], Ty::sort("TyMolecule")),
    ])
    .unwrap()
}

/// Signature for the model environment.
fn environment_signature() -> Signature {
    Signature::parse([
        // Define different cell lines in terms of their components (note that this has currently no effect, i.e. can be considered annotation).
        SignatureDecl::sort("TyCDK1"),
        SignatureDecl::sort("TyCCNB1"),
        SignatureDecl::sort("TyCCNB2"),
        SignatureDecl::sort("CellLine1"),
        SignatureDecl::sort("CellLine2"),
        SignatureDecl::sort("CellLine3"),
        SignatureDecl::sort("CellOmnipotent"),
        SignatureDecl::operation("iota_cl1_cdk1", [Ty::sort("CellLine1")], Ty::sort("TyCDK1")),
        SignatureDecl::operation("iota_cl1_ccnb1", [Ty::sort("CellLine1")], Ty::sort("TyCCNB1")),
        SignatureDecl::operation("iota_cl2_cdk1", [Ty::sort("CellLine2")], Ty::sort("TyCDK1")),
        SignatureDecl::operation("iota_cl2_ccnb2", [Ty::sort("CellLine2")], Ty::sort("TyCCNB2")),
        SignatureDecl::operation("iota_cl3_ccnb1", [Ty::sort("CellLine3")], Ty::sort("TyCCNB1")),
        SignatureDecl::operation("iota_cl3_ccnb2", [Ty::sort("CellLine3")], Ty::sort("TyCCNB2")),
        SignatureDecl::operation(
            "iota_omni_1",
            [Ty::sort("CellOmnipotent")],
            Ty::sort("CellLine1"),
        ),
        SignatureDecl::operation(
            "iota_omni_2",
            [Ty::sort("CellOmnipotent")],
            Ty::sort("CellLine2"),
        ),
        SignatureDecl::operation(
            "iota_omni_3",
            [Ty::sort("CellOmnipotent")],
            Ty::sort("CellLine3"),
        ),
        // Define CellLine12 as coproduct of CellLine1 and CellLine2
        SignatureDecl::sort("CellLine12"),
        SignatureDecl::operation("iota_cl1in12", [Ty::sort("CellLine1")], Ty::sort("CellLine12")),
        SignatureDecl::operation("iota_cl2in12", [Ty::sort("CellLine2")], Ty::sort("CellLine12")),
        SignatureDecl::sort("TyEnv"),
        SignatureDecl::operation("iota_cl12", [Ty::sort("CellLine12")], Ty::sort("TyEnv")),
        SignatureDecl::operation("iota_cl3", [Ty::sort("CellLine3")], Ty::sort("TyEnv")),
    ])
    .unwrap()
}

/// Signature to ground the base in `[]``.
fn ground_base() -> Signature {
    Signature::parse([
        SignatureDecl::sort("Res"),
        SignatureDecl::operation("u", [], Ty::sort("Res")),
        SignatureDecl::operation("p", [], Ty::sort("Res")),
    ])
    .unwrap()
}

/// Signature to ground the environment in `[]`.
fn ground_environment(n: i32) -> Signature {
    Signature::parse([
        SignatureDecl::sort(format!("CellLine{n}")),
        SignatureDecl::operation(format!("!env{n}"), [], Ty::sort(format!("CellLine{n}"))),
    ])
    .unwrap()
}

/// Full signature.
fn signature(n: i32) -> Signature {
    let sig1 = base_signature();
    let sig2 = environment_signature();
    let sig3 = ground_base();
    let sig4 = ground_environment(n);
    merge_signatures(&[sig1, sig2, sig3, sig4])
}

// Declares Model.
fn model_decl() -> [ModelDecl; 3] {
    use crate::surface::*;
    [
        ModelDecl::agent("CDC25", [ObTm::var("r")], [Ty::sort("Res")]),
        ModelDecl::agent("Env", [ObTm::var("e")], [Ty::sort("TyEnv")]),
        // TODO: perhaps we could add a field `dep` to rules for dependencies that occur on both lhs and rhs?
        ModelDecl::rule(
            "phosphorylation",
            [ObTm::var("e")],
            [Ty::sort("CellLine12")],
            PatTm::tensor([
                PatTm::res("Env", [MorTm::var("e")]),
                PatTm::res("CDC25", [MorTm::app("u", [])]),
            ]),
            PatTm::tensor([
                PatTm::res("Env", [MorTm::var("e")]),
                PatTm::res("CDC25", [MorTm::app("p", [])]),
            ]),
        ),
    ]
}

// Generates Model.
fn model(n: i32) -> Model {
    let decls = model_decl();
    Model::parse(signature(n), decls).unwrap()
}

#[test]
fn parse_signature() {
    let expected = expect![[r#"
        #/ sorts:
        TyMolecule
        TyCDK1
        TyCCNB1
        TyCCNB2
        Res
        CellLine1
        CellLine2
        CellLine3
        CellOmnipotent
        CellLine12
        TyEnv
        #/ operations:
        iota_cdk1 : [TyCDK1] → TyMolecule
        iota_ccnb1 : [TyCCNB1] → TyMolecule
        iota_ccnb2 : [TyCCNB2] → TyMolecule
        iota_cl1_cdk1 : [CellLine1] → TyCDK1
        iota_cl1_ccnb1 : [CellLine1] → TyCCNB1
        iota_cl2_cdk1 : [CellLine2] → TyCDK1
        iota_cl2_ccnb2 : [CellLine2] → TyCCNB2
        iota_cl3_ccnb1 : [CellLine3] → TyCCNB1
        iota_cl3_ccnb2 : [CellLine3] → TyCCNB2
        iota_omni_1 : [CellOmnipotent] → CellLine1
        iota_omni_2 : [CellOmnipotent] → CellLine2
        iota_omni_3 : [CellOmnipotent] → CellLine3
        iota_cl1in12 : [CellLine1] → CellLine12
        iota_cl2in12 : [CellLine2] → CellLine12
        iota_cl12 : [CellLine12] → TyEnv
        iota_cl3 : [CellLine3] → TyEnv
        u : [] → Res
        p : [] → Res
        !env1 : [] → CellLine1
    "#]];
    expected.assert_eq(&signature(1).to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        TyMolecule
        TyCDK1
        TyCCNB1
        TyCCNB2
        Res
        CellLine1
        CellLine2
        CellLine3
        CellOmnipotent
        CellLine12
        TyEnv
        #/ operations:
        iota_cdk1 : [TyCDK1] → TyMolecule
        iota_ccnb1 : [TyCCNB1] → TyMolecule
        iota_ccnb2 : [TyCCNB2] → TyMolecule
        iota_cl1_cdk1 : [CellLine1] → TyCDK1
        iota_cl1_ccnb1 : [CellLine1] → TyCCNB1
        iota_cl2_cdk1 : [CellLine2] → TyCDK1
        iota_cl2_ccnb2 : [CellLine2] → TyCCNB2
        iota_cl3_ccnb1 : [CellLine3] → TyCCNB1
        iota_cl3_ccnb2 : [CellLine3] → TyCCNB2
        iota_omni_1 : [CellOmnipotent] → CellLine1
        iota_omni_2 : [CellOmnipotent] → CellLine2
        iota_omni_3 : [CellOmnipotent] → CellLine3
        iota_cl1in12 : [CellLine1] → CellLine12
        iota_cl2in12 : [CellLine2] → CellLine12
        iota_cl12 : [CellLine12] → TyEnv
        iota_cl3 : [CellLine3] → TyEnv
        u : [] → Res
        p : [] → Res
        !env1 : [] → CellLine1
        #/ agents:
        [r] : [Res] ⊢ CDC25 [r]
        [e] : [TyEnv] ⊢ Env [e]
        #/ rules:
        [e] : [CellLine12] ⊢
          phosphorylation [e] : (Env [e], CDC25 [u []]) → (Env [e], CDC25 [p []])
    "#]];
    expected.assert_eq(&model(1).to_string());
}

// Test that reaction fires in CellLine1.
#[test]
fn generate_network_1() {
    use itertools::Itertools;
    let model = model(1);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        CDC25 [u []]
        CDC25 [p []]
        Env [iota_cl12 [iota_cl1in12 [!env1 []]]]"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![[r#"
        phosphorylation [iota_cl1in12 [!env1 []]]
          : (Env [iota_cl1in12 [!env1 []]], CDC25 [u []])
          → (Env [iota_cl1in12 [!env1 []]], CDC25 [p []])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// Test that reaction fires in CellLine2.
#[test]
fn generate_network_2() {
    use itertools::Itertools;
    let model = model(2);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        CDC25 [u []]
        CDC25 [p []]
        Env [iota_cl12 [iota_cl2in12 [!env2 []]]]"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![[r#"
        phosphorylation [iota_cl2in12 [!env2 []]]
          : (Env [iota_cl2in12 [!env2 []]], CDC25 [u []])
          → (Env [iota_cl2in12 [!env2 []]], CDC25 [p []])"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// Test that reaction does not fire in CellLine3.
#[test]
fn generate_network_3() {
    use itertools::Itertools;
    let model = model(3);
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        CDC25 [u []]
        CDC25 [p []]
        Env [iota_cl3 [!env3 []]]"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![""];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}

// TODO: Create example for CellOmnipotent once we can handel product species (issue #9)
