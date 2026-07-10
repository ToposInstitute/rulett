//! https://q.uiver.app/#q=WzAsMTEsWzEsMCwiVHlNb25vbWVyIl0sWzEsMSwiTnQgXFxvdGltZXMgUiBcXG90aW1lcyBDdCJdLFswLDQsIlJfMSJdLFsyLDQsIlJfMiJdLFsxLDMsIlIiXSxbMiwwLCJDdCBcXG90aW1lcyBOdCJdLFsyLDEsIkNOIl0sWzEsNywiU2l0ZVJfMVJfMSJdLFsxLDYsIlJfMSBcXG90aW1lcyBDTiBcXG90aW1lcyBSXzIiXSxbNCw2LCJTaXRlUl8xUl8xIFxcb3RpbWVzIFNpdGVDIl0sWzQsNywiSSJdLFsxLDAsIlxcaW90YV97YW1pbm9hY2lkfSIsMV0sWzIsNCwiXFxpb3RhX3tSXzF9IiwxXSxbMyw0LCJcXGlvdGFfe1JfMn0iLDFdLFs2LDUsImJvbmRfe3BlcHRpZGV9IiwxXSxbNyw4LCJcXGlvdGFfe1NpdGVSXzFSXzF9IiwxXSxbMTAsOSwiYm9uZCIsMV1d

mod common;
use common::*;
use rulett::prelude::name;

/// Base signature.
fn base_signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("TyMonomer"),
        SignatureDecl::sort("Nt"), // Think "Head"
        SignatureDecl::sort("Ct"), // Think "Tail"
        SignatureDecl::sort("R"),  // Think "some amino acid residue"
        SignatureDecl::sort("R1"),
        SignatureDecl::sort("R2"),
        SignatureDecl::sort("CN"),
        SignatureDecl::sort("SiteR1R1"),
        SignatureDecl::sort("SiteNotR1R1"),
        SignatureDecl::sort("SiteC"),
        // Operations
        SignatureDecl::operation(
            "i_aminoacid",
            [Ty::sort("Ct"), Ty::sort("R"), Ty::sort("Nt")],
            Ty::sort("TyMonomer"),
        ),
        SignatureDecl::operation("i_R1", [Ty::sort("R1")], Ty::sort("R")),
        SignatureDecl::operation("i_R2", [Ty::sort("R2")], Ty::sort("R")),
        SignatureDecl::operation(
            // think "CN bond"
            "bond_peptide",
            [Ty::sort("CN")],
            Ty::tensor([Ty::sort("Ct"), Ty::sort("Nt")]),
        ),
        SignatureDecl::operation(
            "i_SiteR1R1",
            [Ty::sort("SiteR1R1")],
            Ty::tensor([Ty::sort("R1"), Ty::sort("CN"), Ty::sort("R1")]),
        ),
        // SignatureDecl::operation(
        // "i_SiteNotR1R1_1",
        // Ty::sort("SiteNotR1R1"),
        // Ty::tensor([Ty::sort("SiteR1"), Ty::sort("CN"), Ty::sort("SiteR2")]),
        // ),
        // SignatureDecl::operation(
        // "i_SiteNotR1R1_2",
        // Ty::sort("SiteNotR1R1"),
        // Ty::tensor([Ty::sort("SiteR2"), Ty::sort("CN"), Ty::sort("SiteR1")]),
        // ),
        // SignatureDecl::operation(
        // "i_SiteNotR1R1_3",
        // Ty::sort("SiteNotR1R1"),
        // Ty::tensor([Ty::sort("SiteR2"), Ty::sort("CN"), Ty::sort("SiteR2")]),
        // ),
    ])
    .unwrap()
}

/// Grounding signature.
fn grounding_signature() -> Signature {
    Signature::parse([
        // Sorts
        SignatureDecl::sort("Nt"), // Think "Head"
        SignatureDecl::sort("Ct"), // Think "Tail"
        SignatureDecl::sort("SiteR1R1"),
        SignatureDecl::sort("SiteNotR1R1"),
        SignatureDecl::sort("SiteC"),
        // Operations
        SignatureDecl::operation("e_N", [], Ty::sort("Nt")),
        SignatureDecl::operation("e_C", [], Ty::sort("Ct")),
        SignatureDecl::operation("e_SiteR1R1", [], Ty::sort("SiteR1R1")),
        SignatureDecl::operation("e_SitenotR1R1", [], Ty::sort("SiteNotR1R1")),
        SignatureDecl::operation("e_SiteC", [], Ty::sort("SiteC")),
        SignatureDecl::operation(
            // SiteR1R1 is an emergent site that can bind to SiteC
            "bond",
            [],
            Ty::tensor([Ty::sort("SiteR1R1"), Ty::sort("SiteC")]),
        ),
    ])
    .unwrap()
}

/// Full signature.
fn signature() -> Signature {
    let sig1 = base_signature();
    let sig2 = grounding_signature();
    merge_signatures(&[sig1, sig2])
}

// Declares Model.
fn model_decl() -> [ModelDecl; 4] {
    use crate::surface::*;
    let polymer = PatTm::let_(
        [ObTm::var("s1"), ObTm::var("s2")],
        MorTm::app("bond_peptide", [MorTm::var("cn")]),
        PatTm::tensor([
            PatTm::res(
                "M",
                [MorTm::app("i_aminoacid", [MorTm::var("n"), MorTm::var("r"), MorTm::var("s1")])],
            ),
            PatTm::res(
                "M",
                [MorTm::app("i_aminoacid", [MorTm::var("s2"), MorTm::var("r'"), MorTm::var("c")])],
            ),
        ]),
    );
    let dimer = PatTm::let_(
        [ObTm::var("cn"), ObTm::var("s4")],
        MorTm::app("bond", []),
        PatTm::tensor([polymer.clone(), PatTm::res("C", [MorTm::var("s4")])]),
    );
    [
        ModelDecl::agent("M", [ObTm::var("m")], [Ty::sort("TyMonomer")]),
        ModelDecl::agent("C", [ObTm::var("c")], [Ty::sort("SiteC")]),
        // TODO: perhaps we could add a field `dep` to rules for dependencies that occur on both lhs and rhs?
        ModelDecl::rule(
            "polymerization",
            [ObTm::var("c"), ObTm::var("r"), ObTm::var("r'"), ObTm::var("n"), ObTm::var("cn")],
            [Ty::sort("Ct"), Ty::sort("R"), Ty::sort("R"), Ty::sort("Nt"), Ty::sort("CN")],
            PatTm::tensor([
                PatTm::res(
                    "M",
                    [MorTm::app(
                        "i_aminoacid",
                        [MorTm::var("n"), MorTm::var("r"), MorTm::app("e_C", [])],
                    )],
                ),
                PatTm::res(
                    "M",
                    [MorTm::app(
                        "i_aminoacid",
                        [MorTm::app("e_N", []), MorTm::var("r"), MorTm::var("c")],
                    )],
                ),
            ]),
            polymer.clone(),
        ), // Note that `polymer` contains the emergent site SiteR1R1
        ModelDecl::rule(
            "dimerization",
            [ObTm::var("n"), ObTm::var("c")],
            [Ty::sort("Nt"), Ty::sort("Ct")],
            PatTm::tensor([
                polymer.subst(&mut vec![(name("cn"), MorTm::app("e_SiteR1R1", []))]),
                PatTm::res("C", [MorTm::app("e_SiteC", [])]),
            ]),
            dimer,
        ),
    ]
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
        TyMonomer
        Nt
        Ct
        R
        R1
        R2
        CN
        SiteR1R1
        SiteNotR1R1
        SiteC
        #/ operations:
        i_aminoacid : [Ct, R, Nt] → TyMonomer
        i_R1 : [R1] → R
        i_R2 : [R2] → R
        bond_peptide : [CN] → ⊗ [Ct, Nt]
        i_SiteR1R1 : [SiteR1R1] → ⊗ [R1, CN, R1]
        e_N : [] → Nt
        e_C : [] → Ct
        e_SiteR1R1 : [] → SiteR1R1
        e_SitenotR1R1 : [] → SiteNotR1R1
        e_SiteC : [] → SiteC
        bond : [] → ⊗ [SiteR1R1, SiteC]
    "#]];
    expected.assert_eq(&signature().to_string());
}

#[test]
fn parse_model() {
    let expected = expect![[r#"
        #/ sorts:
        TyMonomer
        Nt
        Ct
        R
        R1
        R2
        CN
        SiteR1R1
        SiteNotR1R1
        SiteC
        #/ operations:
        i_aminoacid : [Ct, R, Nt] → TyMonomer
        i_R1 : [R1] → R
        i_R2 : [R2] → R
        bond_peptide : [CN] → ⊗ [Ct, Nt]
        i_SiteR1R1 : [SiteR1R1] → ⊗ [R1, CN, R1]
        e_N : [] → Nt
        e_C : [] → Ct
        e_SiteR1R1 : [] → SiteR1R1
        e_SitenotR1R1 : [] → SiteNotR1R1
        e_SiteC : [] → SiteC
        bond : [] → ⊗ [SiteR1R1, SiteC]
        #/ agents:
        [m] : [TyMonomer] ⊢ M [m]
        [c] : [SiteC] ⊢ C [c]
        #/ rules:
        [c, r, r', n, cn] : [Ct, R, R, Nt, CN] ⊢
          polymerization [c, r, r', n, cn]
            : (M [i_aminoacid [n, r, e_C []]], M [i_aminoacid [e_N [], r, c]])
            → let bond_peptide [cn] in
              (M [i_aminoacid [n, r, 0.0]], M [i_aminoacid [0.1, r', c]])
        [n, c] : [Nt, Ct] ⊢
          dimerization [n, c]
            : (
              let bond_peptide [e_SiteR1R1 []] in
                (M [i_aminoacid [n, r, 0.0]], M [i_aminoacid [0.1, r', c]]),
              C [e_SiteC []]
            )
            → let bond [] in
              (
                let bond_peptide [0.0] in
                  (M [i_aminoacid [n, r, 0.0]], M [i_aminoacid [0.1, r', c]]),
                C [0.1]
              )
    "#]];
    expected.assert_eq(&model().to_string());
}

// Test that four dimerization and one polymerization reaction are created
#[test]
fn generate_network() {
    use itertools::Itertools;
    let model = model();
    let generator = NetGenerator::new(&model);

    let species = expect![[r#"
        C [e_SiteC []]
        let i_SiteR1R1 [e_SiteR1R1 []] in
          let bond_peptide [0.1] in
            (
              M [i_aminoacid [e_C [], i_R1 [1.0], e_N []]],
              M [i_aminoacid [0.0, i_R1 [1.2], 0.1]]
            )
        let i_SiteR1R1 [e_SiteR1R1 []] in
          let bond_peptide [0.1] in
            (
              M [i_aminoacid [e_C [], i_R1 [1.0], 0.1]],
              M [i_aminoacid [0.0, i_R1 [1.2], e_N []]]
            )
        let i_SiteR1R1 [e_SiteR1R1 []] in
          let bond_peptide [0.1] in
            (
              M [i_aminoacid [0.0, i_R1 [1.0], e_N []]],
              M [i_aminoacid [e_C [], i_R1 [1.2], 0.1]]
            )
        let i_SiteR1R1 [e_SiteR1R1 []] in
          let bond_peptide [0.1] in
            (
              M [i_aminoacid [0.0, i_R1 [1.0], 0.1]],
              M [i_aminoacid [e_C [], i_R1 [1.2], e_N []]]
            )"#]];
    species.assert_eq(&generator.species(2).join("\n"));

    let transitions = expect![[r#"
        let i_SiteR1R1 [e_SiteR1R1 []] in
            polymerization [e_C [], i_R1 [0.0], i_R1 [0.2], e_N [], 0.1]
          : let i_SiteR1R1 [e_SiteR1R1 []] in
            (
              M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]],
              M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]]
            )
          → let i_SiteR1R1 [e_SiteR1R1 []] in
            let bond_peptide [0.1] in
              (
                M [i_aminoacid [e_N [], i_R1 [1.0], 0.0]],
                M [i_aminoacid [0.1, i_R1 [1.2], e_C []]]
              )
        dimerization [e_N [], e_C []]
          : (
            let bond_peptide [e_SiteR1R1 []] in
              (M [i_aminoacid [e_N [], r, 0.0]], M [i_aminoacid [0.1, r', e_C []]]),
            C [e_SiteC []]
          )
          → let bond [] in
            (
              let bond_peptide [0.0] in
                (M [i_aminoacid [e_N [], r, 0.0]], M [i_aminoacid [0.1, r', e_C []]]),
              C [0.1]
            )
        let bond [] in
            let i_SiteR1R1 [0.0] in
              (C [1.1], polymerization [e_C [], i_R1 [0.0], i_R1 [0.2], e_N [], 0.1])
          : let bond [] in
            let i_SiteR1R1 [0.0] in
              (
                C [1.1],
                (
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]]
                )
              )
          → let bond [] in
            let i_SiteR1R1 [0.0] in
              (
                C [1.1],
                let bond_peptide [0.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [1.2], e_C []]]
                  )
              )
        let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                polymerization [e_C [], i_R1 [0.0], i_R1 [1.0], e_N [], 0.1],
                polymerization [e_C [], i_R1 [0.2], i_R1 [1.2], e_N [], 1.1]
              )
          : let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                (
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]]
                ),
                (
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]]
                )
              )
          → let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                let bond_peptide [0.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.0], e_C []]]
                  ),
                let bond_peptide [1.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.2], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.2], e_C []]]
                  )
              )
        let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                polymerization [e_C [], i_R1 [1.0], i_R1 [0.0], e_N [], 0.1],
                polymerization [e_C [], i_R1 [0.2], i_R1 [1.2], e_N [], 1.1]
              )
          : let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                (
                  M [i_aminoacid [e_N [], i_R1 [1.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [1.0], e_C []]]
                ),
                (
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]]
                )
              )
          → let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                let bond_peptide [0.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [2.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [1.0], e_C []]]
                  ),
                let bond_peptide [1.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.2], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.2], e_C []]]
                  )
              )
        let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                polymerization [e_C [], i_R1 [0.0], i_R1 [0.2], e_N [], 1.1],
                polymerization [e_C [], i_R1 [1.0], i_R1 [1.2], e_N [], 0.1]
              )
          : let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                (
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]]
                ),
                (
                  M [i_aminoacid [e_N [], i_R1 [1.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [1.0], e_C []]]
                )
              )
          → let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                let bond_peptide [1.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [1.2], e_C []]]
                  ),
                let bond_peptide [0.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [2.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.2], e_C []]]
                  )
              )
        let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                polymerization [e_C [], i_R1 [0.0], i_R1 [1.0], e_N [], 1.1],
                polymerization [e_C [], i_R1 [0.2], i_R1 [1.2], e_N [], 0.1]
              )
          : let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                (
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.0], e_C []]]
                ),
                (
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]]
                )
              )
          → let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                let bond_peptide [1.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.0], e_C []]]
                  ),
                let bond_peptide [0.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.2], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.2], e_C []]]
                  )
              )
        let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                polymerization [e_C [], i_R1 [1.0], i_R1 [0.0], e_N [], 1.1],
                polymerization [e_C [], i_R1 [0.2], i_R1 [1.2], e_N [], 0.1]
              )
          : let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                (
                  M [i_aminoacid [e_N [], i_R1 [1.0], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [1.0], e_C []]]
                ),
                (
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]],
                  M [i_aminoacid [e_N [], i_R1 [0.2], e_C []]]
                )
              )
          → let i_SiteR1R1 [e_SiteR1R1 []] in
            let i_SiteR1R1 [e_SiteR1R1 []] in
              (
                let bond_peptide [1.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [2.0], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [1.0], e_C []]]
                  ),
                let bond_peptide [0.1] in
                  (
                    M [i_aminoacid [e_N [], i_R1 [1.2], 0.0]],
                    M [i_aminoacid [0.1, i_R1 [2.2], e_C []]]
                  )
              )"#]];
    transitions.assert_eq(&generator.transitions(2).join("\n"));
}
