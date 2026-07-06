//! Organic chemistry example with
//! - halogen-halogen exchange rule
//! - hydrogen-hydrogen exchange rule
//!
//! described by
//! https://q.uiver.app/#q=WzAsMjcsWzEsNSwiXFx0ZXh0e05hfV4rIDogXFx0ZXh0e1Bvc30iXSxbMSw3LCJcXHRleHR7SH1eLSA6IFxcdGV4dHtOZWd9Il0sWzIsNSwiXFxtYXRocm17SGFsb31eLSA6IFxcdGV4dHtOZWd9IFxcb3RpbWVzIFxcdGV4dHtUeUhhbG99Il0sWzIsMCwiXFx0ZXh0e1R5SGFsb30iXSxbMiwxLCJJIl0sWzMsMSwiSSJdLFswLDAsIlxcdGV4dGJme1R5cGVzfToiXSxbMSwwLCJcXHRleHR7UG9zfSJdLFsxLDEsIlxcdGV4dHtOZWd9Il0sWzAsNSwiXFx0ZXh0YmZ7QWdlbnRzfToiXSxbMiwzLCJcXHRleHR7UG9zfSBcXG90aW1lcyBcXHRleHR7TmVnfSJdLFsxLDQsIkkiXSxbMiw0LCJcXHRleHR7UmFkfSBcXG90aW1lcyBcXHRleHR7UmFkfSJdLFsyLDcsIlxcdGV4dHtDSH1fM14tIDogXFx0ZXh0e05lZ30iXSxbNCw1LCJQIl0sWzUsNiwiUCBcXGNvbG9uZXFxIChcXHRleHR7Q0h9XzNeXFxidWxsZXQgXFxvdGltZXMgXFx0ZXh0e0hhbG99XlxcYnVsbGV0KShcXHRleHR7Ym9uZH1fYyBcXG90aW1lcyAxKSBcXG90aW1lcyAoXFx0ZXh0e05hfV4rIFxcb3RpbWVzIFxcdGV4dHtIYWxvfV4tKShcXHRleHR7Ym9uZH1faSBcXG90aW1lcyAxKSJdLFs1LDUsIlAoXFxzaWdtYV97MiwxfSkgOiBcXHRleHR7VHlIYWxvfV57XFxvdGltZXMgMn0iXSxbNCw2LCJcXHRleHR7d2hlcmV9Il0sWzQsOCwiKFxcdGV4dHtDSH1fM15cXGJ1bGxldCBcXG90aW1lcyBcXHRleHR7SGFsb31eXFxidWxsZXQpKFxcdGV4dHtib25kfV9jIFxcb3RpbWVzIDEpIFxcb3RpbWVzIChcXHRleHR7TmF9XisgXFxvdGltZXMgXFx0ZXh0e0h9Xi0pKFxcdGV4dHtib25kfV9pKSJdLFs1LDgsIihcXHRleHR7Q0h9XzNeXFxidWxsZXQgXFxvdGltZXMgXFx0ZXh0e0h9XlxcYnVsbGV0KShcXHRleHR7Ym9uZH1fYykgXFxvdGltZXMgKFxcdGV4dHtOYX1eKyBcXG90aW1lcyBcXHRleHR7SGFsb31eLSkoXFx0ZXh0e2JvbmR9X2kgXFxvdGltZXMgMSk6IFxcdGV4dHtUeUhhbG99Il0sWzQsNCwiXFx0ZXh0YmZ7UnVsZXN9OiJdLFsxLDMsIkkiXSxbMiw4LCJcXHRleHR7Q0h9XzNeXFxidWxsZXQgOiBcXHRleHR7UmFkfSJdLFszLDQsIlxcdGV4dHsoY29tbXV0YXRpdmUpfSJdLFsxLDIsIlxcdGV4dHtSYWR9Il0sWzIsNiwiXFx0ZXh0e0hhbG99XlxcYnVsbGV0IDogXFx0ZXh0e1JhZH0gXFxvdGltZXMgXFx0ZXh0e1R5SGFsb30iXSxbMSw4LCJcXHRleHR7SH1eXFxidWxsZXQgOiBcXHRleHR7UmFkfSJdLFs0LDMsIlxcdGV4dHtGfSJdLFs1LDMsIlxcdGV4dHtDbH0iLDJdLFsxMSwxMiwiXFx0ZXh0e2JvbmR9X2MiXSxbMTQsMTYsIlJfMSJdLFsxOCwxOSwiUl8yIl0sWzIxLDEwLCJcXHRleHR7Ym9uZH1faSJdXQ==

mod common;
use common::*;

// Signature
fn organic_chemistry_signature() -> Signature {
    Signature::parse([
        SignatureDecl::sort("Pos"),
        SignatureDecl::sort("Neg"),
        SignatureDecl::sort("Rad"),
        SignatureDecl::sort("TyHalo"),
        SignatureDecl::operation("F", [], Ty::sort("TyHalo")),
        SignatureDecl::operation("Cl", [], Ty::sort("TyHalo")),
        SignatureDecl::operation("bond_i", [], Ty::tensor([Ty::sort("Pos"), Ty::sort("Neg")])),
        SignatureDecl::operation("bond_c", [], Ty::tensor([Ty::sort("Rad"), Ty::sort("Rad")])),
    ])
    .unwrap()
}

// Declares model.
fn organic_chemistry_model_decls() -> [ModelDecl; 9] {
    use surface::*;
    [
        ModelDecl::agent("Na", [ObTm::var("p")], [Ty::sort("Pos")]),
        ModelDecl::agent("H_minus", [ObTm::var("n")], [Ty::sort("Neg")]),
        ModelDecl::agent("H_dot", [ObTm::var("r")], [Ty::sort("Rad")]),
        ModelDecl::agent("CH3_minus", [ObTm::var("n")], [Ty::sort("Neg")]),
        ModelDecl::agent("CH3_dot", [ObTm::var("r")], [Ty::sort("Rad")]),
        ModelDecl::agent(
            "Halo_neg",
            [ObTm::var("n"), ObTm::var("h")],
            [Ty::sort("Neg"), Ty::sort("TyHalo")],
        ),
        ModelDecl::agent(
            "Halo_dot",
            [ObTm::var("r"), ObTm::var("h")],
            [Ty::sort("Rad"), Ty::sort("TyHalo")],
        ),
        // Halogen-halogen exchange
        ModelDecl::rule(
            "rule1",
            [ObTm::var("h1"), ObTm::var("h2")],
            [Ty::sort("TyHalo"), Ty::sort("TyHalo")],
            PatTm::tensor([
                PatTm::let_(
                    ObTm::tensor([ObTm::var("r1"), ObTm::var("r2")]),
                    MorTm::app("bond_c", []),
                    PatTm::tensor([
                        PatTm::res("CH3_dot", [MorTm::var("r1")]),
                        PatTm::res("Halo_dot", [MorTm::var("r2"), MorTm::var("h1")]),
                    ]),
                ),
                PatTm::let_(
                    ObTm::tensor([ObTm::var("p"), ObTm::var("n")]),
                    MorTm::app("bond_i", []),
                    PatTm::tensor([
                        PatTm::res("Na", [MorTm::var("p")]),
                        PatTm::res("Halo_dot", [MorTm::var("n"), MorTm::var("h2")]),
                    ]),
                ),
            ]),
            PatTm::tensor([
                PatTm::let_(
                    ObTm::tensor([ObTm::var("r1"), ObTm::var("r2")]),
                    MorTm::app("bond_c", []),
                    PatTm::tensor([
                        PatTm::res("CH3_dot", [MorTm::var("r1")]),
                        PatTm::res("Halo_dot", [MorTm::var("r2"), MorTm::var("h2")]),
                    ]),
                ),
                PatTm::let_(
                    ObTm::tensor([ObTm::var("p"), ObTm::var("n")]),
                    MorTm::app("bond_i", []),
                    PatTm::tensor([
                        PatTm::res("Na", [MorTm::var("p")]),
                        PatTm::res("Halo_dot", [MorTm::var("n"), MorTm::var("h1")]),
                    ]),
                ),
            ]),
        ),
        // Halogen-hydrogen exchange
        ModelDecl::rule(
            "rule2",
            [ObTm::var("h1")],
            [Ty::sort("TyHalo")],
            PatTm::tensor([
                PatTm::let_(
                    ObTm::tensor([ObTm::var("r1"), ObTm::var("r2")]),
                    MorTm::app("bond_c", []),
                    PatTm::tensor([
                        PatTm::res("CH3_dot", [MorTm::var("r1")]),
                        PatTm::res("Halo_dot", [MorTm::var("r2"), MorTm::var("h1")]),
                    ]),
                ),
                PatTm::let_(
                    ObTm::tensor([ObTm::var("p"), ObTm::var("n")]),
                    MorTm::app("bond_i", []),
                    PatTm::tensor([
                        PatTm::res("Na", [MorTm::var("p")]),
                        PatTm::res("H_minus", [MorTm::var("n")]),
                    ]),
                ),
            ]),
            PatTm::tensor([
                PatTm::let_(
                    ObTm::tensor([ObTm::var("r1"), ObTm::var("r2")]),
                    MorTm::app("bond_c", []),
                    PatTm::tensor([
                        PatTm::res("CH3_dot", [MorTm::var("r1")]),
                        PatTm::res("H_dot", [MorTm::var("r2")]),
                    ]),
                ),
                PatTm::let_(
                    ObTm::tensor([ObTm::var("p"), ObTm::var("n")]),
                    MorTm::app("bond_i", []),
                    PatTm::tensor([
                        PatTm::res("Na", [MorTm::var("p")]),
                        PatTm::res("Halo_minus", [MorTm::var("n")]),
                    ]),
                ),
            ]),
        ),
    ]
}

// Generates model.
fn organic_chemistry_model() -> Model {
    let decls = organic_chemistry_model_decls();
    Model::parse(organic_chemistry_signature(), decls).unwrap()
}

#[test]
fn parse() {
    // toy_model_chem
    let expected = expect![[r#"
        #/ sorts:
        Pos
        Neg
        Rad
        TyHalo
        #/ operations:
        F : [] → TyHalo
        Cl : [] → TyHalo
        bond_i : [] → ⊗ [Pos, Neg]
        bond_c : [] → ⊗ [Rad, Rad]
        #/ agents:
        [p] : [Pos] ⊢ Na [p]
        [n] : [Neg] ⊢ H_minus [n]
        [r] : [Rad] ⊢ H_dot [r]
        [n] : [Neg] ⊢ CH3_minus [n]
        [r] : [Rad] ⊢ CH3_dot [r]
        [n, h] : [Neg, TyHalo] ⊢ Halo_neg [n, h]
        [r, h] : [Rad, TyHalo] ⊢ Halo_dot [r, h]
        #/ rules:
        [h1, h2] : [TyHalo, TyHalo] ⊢
          rule1 [h1, h2]
            : (
              let bond_c [] in (CH3_dot [0.0], Halo_dot [0.1, h1]),
              let bond_i [] in (Na [0.0], Halo_dot [0.1, h2])
            )
            → (
              let bond_c [] in (CH3_dot [0.0], Halo_dot [0.1, h2]),
              let bond_i [] in (Na [0.0], Halo_dot [0.1, h1])
            )
        [h1] : [TyHalo] ⊢
          rule2 [h1]
            : (
              let bond_c [] in (CH3_dot [0.0], Halo_dot [0.1, h1]),
              let bond_i [] in (Na [0.0], H_minus [0.1])
            )
            → (
              let bond_c [] in (CH3_dot [0.0], H_dot [0.1]),
              let bond_i [] in (Na [0.0], Halo_minus [0.1])
            )
    "#]];
    expected.assert_eq(&organic_chemistry_model().to_string());
}

#[test]
#[ignore = "Waiting for bug fix in issue #7 and unifying `bond_c [] in (Halo_dot [r#1, F []], Halo_dot [r#2, Cl []]` with `bond_c [] in (Halo_dot [r#1, Cl []], Halo_dot [r#2, F []]`"]
fn netgen() {
    let model = organic_chemistry_model();
    let generator = NetGenerator::new(&model);

    let net = expect![[r#"
        #/ species:
        let (p, n) = bond_i [] in (Na [p], H_minus [n])
        let (p, n) = bond_i [] in (Na [p], CH3_minus [n])
        let (p, n) = bond_i [] in (Na [p], Halo_neg [n, F []])
        let (p, n) = bond_i [] in (Na [p], Halo_neg [n, Cl []])
        let (r#1, r#2) = bond_c [] in (H_dot [r#1], H_dot [r#2])
        let (r#1, r#2) = bond_c [] in (H_dot [r#1], CH3_dot [r#2])
        let (r#1, r#2) = bond_c [] in (H_dot [r#1], Halo_dot [r#2, F []])
        let (r#1, r#2) = bond_c [] in (H_dot [r#1], Halo_dot [r#2, Cl []])
        let (r#1, r#2) = bond_c [] in (CH3_dot [r#1], CH3_dot [r#2])
        let (r#1, r#2) = bond_c [] in (CH3_dot [r#1], Halo_dot [r#2, F []])
        let (r#1, r#2) = bond_c [] in (CH3_dot [r#1], Halo_dot [r#2, Cl []])
        let (r#1, r#2) = bond_c [] in (Halo_dot [r#1, F []], Halo_dot [r#2, F []])
        let (r#1, r#2) = bond_c [] in (Halo_dot [r#1, F []], Halo_dot [r#2, Cl []])
        let (r#1, r#2) = bond_c [] in (Halo_dot [r#1, Cl []], Halo_dot [r#2, F []])
        let (r#1, r#2) = bond_c [] in (Halo_dot [r#1, Cl []], Halo_dot [r#2, Cl []])
        #/ transitions:
        rule1 [F [], F []]
            : (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, F []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, F []])
            )
            → (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, F []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, F []])
            )
        rule1 [F [], Cl []]
            : (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, F []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, Cl []])
            )
            → (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, Cl []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, F []])
            )
        rule1 [Cl [], F []]
            : (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, Cl []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, F []])
            )
            → (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, F []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, Cl []])
            )
        rule1 [Cl [], Cl []]
            : (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, Cl []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, Cl []])
            )
            → (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, Cl []]),
            let [p, n] = bond_i [] in (Na [p], Halo_dot [n, Cl []])
            )
        rule2 [F []]
            : (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, F []]),
            let [p, n] = bond_i [] in (Na [p], H_minus [n])
            )
            → (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], H_dot [r2]),
            let [p, n] = bond_i [] in (Na [p], Halo_minus [n])
            )
        rule2 [Cl []]
            : (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], Halo_dot [r2, Cl []]),
            let [p, n] = bond_i [] in (Na [p], H_minus [n])
            )
            → (
            let [r1, r2] = bond_c [] in (CH3_dot [r1], H_dot [r2]),
            let [p, n] = bond_i [] in (Na [p], Halo_minus [n])
            )"#]];
    // let species = expect![[r#"
    // let bond_i [] in (Na [0.0], H_minus [0.1])
    // let bond_i [] in (Na [0.0], CH3_minus [0.1])
    // let bond_i [] in (Na [0.0], Halo_neg [0.1, F []])
    // let bond_i [] in (Na [0.0], Halo_neg [0.1, Cl []])
    // let bond_c [] in (H_dot [0.0], H_dot [0.1])
    // let bond_c [] in (H_dot [0.0], CH3_dot [0.1])
    // let bond_c [] in (H_dot [0.0], Halo_dot [0.1, F []])
    // let bond_c [] in (H_dot [0.0], Halo_dot [0.1, Cl []])
    // let bond_c [] in (CH3_dot [0.0], CH3_dot [0.1])
    // let bond_c [] in (CH3_dot [0.0], Halo_dot [0.1, F []])
    // let bond_c [] in (CH3_dot [0.0], Halo_dot [0.1, Cl []])
    // let bond_c [] in (Halo_dot [0.0, F []], Halo_dot [0.1, F []])
    // let bond_c [] in (Halo_dot [0.0, F []], Halo_dot [0.1, Cl []])
    // let bond_c [] in (Halo_dot [0.0, Cl []], Halo_dot [0.1, Cl []])"#]];
    // species.assert_eq(&generator.species(2).join("\n"));
    net.assert_eq(&generator.net(2).to_string());
}
