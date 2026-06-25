mod common;
use common::*;

fn phospho_tyrosine_signature() -> Signature {
    Signature::parse([
        SignatureDecl::sort("Tyr"),
        SignatureDecl::sort("SH2"),
        SignatureDecl::sort("xTyr"),
        SignatureDecl::operation("e_sh2", [], Ty::sort("SH2")),
        SignatureDecl::operation("e_xtyr", [], Ty::sort("xTyr")),
        SignatureDecl::operation("u", [Ty::sort("xTyr")], Ty::sort("Tyr")),
        SignatureDecl::operation("p", [Ty::sort("xTyr")], Ty::sort("Tyr")),
        SignatureDecl::operation("bond", [], Ty::tensor([Ty::sort("SH2"), Ty::sort("xTyr")])),
    ])
    .unwrap()
}

fn phospho_tyrosine_model_decls() -> [ModelDecl; 4] {
    use surface::*;
    [
        ModelDecl::agent("A", [ObTm::var("x")], [Ty::sort("SH2")]),
        ModelDecl::agent("C", [ObTm::var("y")], [Ty::sort("Tyr")]),
        ModelDecl::rule(
            "R_phosphorylation",
            [],
            [],
            PatTm::res("C", [MorTm::app("u", [MorTm::app("e_xtyr", [])])]),
            PatTm::res("C", [MorTm::app("p", [MorTm::app("e_xtyr", [])])]),
        ),
        ModelDecl::rule(
            "R_dimerization",
            [],
            [],
            PatTm::tensor([
                PatTm::res("A", [MorTm::app("e_sh2", [])]),
                PatTm::res("C", [MorTm::app("p", [MorTm::app("e_xtyr", [])])]),
            ]),
            PatTm::let_(
                ObTm::tensor([ObTm::var("s1"), ObTm::var("s2")]),
                MorTm::app("bond", []),
                PatTm::tensor([
                    PatTm::res("A", [MorTm::var("s1")]),
                    PatTm::res("C", [MorTm::app("p", [MorTm::var("s2")])]),
                ]),
            ),
        ),
    ]
} // TODO: Implement this model as preorder (currently `u` and `p` are parallel morphisms); will require enableing LHS and RHS mismatches for phosphorylation rule

fn phospho_tyrosine_model() -> Model {
    let decls = phospho_tyrosine_model_decls();
    Model::parse(phospho_tyrosine_signature(), decls).unwrap()
}

#[test]
fn parse() {
    // toy_model_chem
    let expected = expect![[r#"
        #/ sorts:
        Tyr
        SH2
        xTyr
        #/ operations:
        e_sh2 : [] → SH2
        e_xtyr : [] → xTyr
        u : [xTyr] → Tyr
        p : [xTyr] → Tyr
        bond : [] → ⊗ [SH2, xTyr]
        #/ agents:
        [x] : [SH2] ⊢ A [x]
        [y] : [Tyr] ⊢ C [y]
        #/ rules:
        [] : [] ⊢ R_phosphorylation [] : C [u [e_xtyr []]] → C [p [e_xtyr []]]
        [] : [] ⊢
          R_dimerization []
            : (A [e_sh2 []], C [p [e_xtyr []]])
            → let bond [] in (A [0.0], C [p [0.1]])
    "#]];
    expected.assert_eq(&phospho_tyrosine_model().to_string());
}

#[test]
fn netgen() {
    let model = phospho_tyrosine_model();
    let generator = NetGenerator::new(&model);

    let net = expect![[r#"
        #/ species:
        A [e_sh2 []]
        C [u [e_xtyr []]]
        C [p [e_xtyr []]]
        let bond [] in (A [0.0], C [u [0.1]])
        let bond [] in (A [0.0], C [p [0.1]])
        #/ transitions:
        R_phosphorylation [] : [C [u [e_xtyr []]]] → [C [p [e_xtyr []]]]
        R_dimerization []
          : [A [e_sh2 []], C [p [e_xtyr []]]]
          → [let bond [] in (A [0.0], C [p [0.1]])]
    "#]];
    net.assert_eq(&generator.net(2).to_string());
}
