//! Rule-based models.

use itertools::zip_eq;
use pretty::RcDoc;
use std::fmt;

use super::{prelude::*, theory::*, tm::*, ty::*};

/// Declaration in the definition of a rule-based model.
pub enum ModelDecl {
    /// Declaration of an agent.
    ///
    /// The variable names defined by the [`ObTm`] are logically superfluous,
    /// but are included as a form of documentation and to be consistent with
    /// rule declarations, where they are necessary.
    Agent { name: Name, interface: (ObTm, Ty) },

    /// Declaration of a basic rule.
    Rule {
        name: Name,
        interface: (ObTm, Ty),
        lhs: PatTm,
        rhs: PatTm,
    },
}

impl ModelDecl {
    /// Smart constructor for [`Agent`](Self::Agent) variant.
    pub fn agent(name: impl Into<Name>, tm: impl Into<ObTm>, ty: impl Into<Ty>) -> Self {
        Self::Agent {
            name: name.into(),
            interface: (tm.into(), ty.into()),
        }
    }

    /// Smart constructor for [`Rule`](Self::Rule) variant.
    pub fn rule(
        name: impl Into<Name>,
        tm: impl Into<ObTm>,
        ty: impl Into<Ty>,
        lhs: impl Into<PatTm>,
        rhs: impl Into<PatTm>,
    ) -> Self {
        Self::Rule {
            name: name.into(),
            interface: (tm.into(), ty.into()),
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
    }
}

/// A rule-based model.
pub struct Model {
    signature: Signature,
    agents: IndexMap<Name, ObTmJudgment>,
    rules: IndexMap<Name, BasicRuleData>,
}

struct BasicRuleData {
    interface: ObTmJudgment,
    lhs: PatTm,
    rhs: PatTm,
}

impl Model {
    /// Constructs an empty model over a signature.
    pub fn new(signature: Signature) -> Self {
        Self {
            signature,
            agents: Default::default(),
            rules: Default::default(),
        }
    }

    /// Gets the signature underlying the model.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Is there an agent with the given name in the model?
    pub fn has_agent(&self, name: &Name) -> bool {
        self.agents.contains_key(name)
    }

    /// Is there a basic rule with the given name in the model?
    pub fn has_rule(&self, name: &Name) -> bool {
        self.rules.contains_key(name)
    }

    /// Iterates over the names of the agents in the model.
    pub fn agent_names(&self) -> impl Iterator<Item = Name> {
        self.agents.keys().copied()
    }

    /// Iterates over the names of the basic rules in the model.
    pub fn rule_names(&self) -> impl Iterator<Item = Name> {
        self.rules.keys().copied()
    }

    /// Gets the interface of an agent or rule in the model.
    pub fn interface(&self, name: &Name) -> Option<&ObTmJudgment> {
        self.agents
            .get(name)
            .or_else(|| self.rules.get(name).map(|data| &data.interface))
    }

    /// Parses a model from a signature and a list of declarations.
    ///
    /// If a model is returned, it is guaranteed to be valid; otherwise, the
    /// first error encountered is reported.
    pub fn parse(
        signature: Signature,
        decls: impl IntoIterator<Item = ModelDecl>,
    ) -> Result<Self, String> {
        let mut model = Self::new(signature);
        for decl in decls {
            model.declare(decl)?;
        }
        Ok(model)
    }

    /// Adds a declaration to the model.
    pub fn declare(&mut self, decl: ModelDecl) -> Result<(), String> {
        match decl {
            ModelDecl::Agent { name, interface: (tm, ty) } => self
                .add_agent(name, tm, ty)
                .map_err(|err| format!("cannot declare agent {name}: {err}")),
            ModelDecl::Rule { name, interface: (tm, ty), lhs, rhs } => self
                .add_rule(name, tm, ty, lhs, rhs)
                .map_err(|err| format!("cannot declare rule {name}: {err}")),
        }
    }

    /// Adds an agent with the given name and interface to the model.
    pub fn add_agent(&mut self, name: Name, tm: ObTm, ty: Ty) -> Result<(), String> {
        let interface = self.check_interface(tm, ty)?;
        if self.has_rule(&name) || self.agents.insert(name, interface).is_some() {
            return Err(format!("{name} already defined"));
        }
        Ok(())
    }

    /// Adds a basic rule to the model.
    pub fn add_rule(
        &mut self,
        name: Name,
        tm: ObTm,
        ty: Ty,
        lhs: PatTm,
        rhs: PatTm,
    ) -> Result<(), String> {
        let interface = self.check_interface(tm, ty)?;
        // TODO: Type check left- and right-hand sides!
        let data = BasicRuleData { interface, lhs, rhs };
        if self.has_agent(&name) || self.rules.insert(name, data).is_some() {
            return Err(format!("{name} already defined"));
        }
        Ok(())
    }

    /// Checks that interface of agent or rule is well-typed.
    fn check_interface(&self, tm: ObTm, ty: Ty) -> Result<ObTmJudgment, String> {
        self.signature
            .check_ty(&ty, &Kind::list(Kind::prim()))
            .map_err(|err| format!("interface has invalid type: {err}"))
            .and_then(|ok| {
                if ok {
                    ObTmJudgment::judge(tm, ty).map_err(|err| format!("ill-typed interface: {err}"))
                } else {
                    Err(format!("interface type should be a list of sorts, received: {ty}"))
                }
            })
    }

    /// Constructs a pattern term corresponding to an agent.
    pub(crate) fn agent_tm(&self, name: Name, terms: Vec<MorTm>) -> PatTm {
        PatTm::Res(name, MorTm::List(terms))
    }

    /// Constructs a rule term corresponding to a basic rule.
    pub(crate) fn rule_tm(&self, name: Name, terms: Vec<MorTm>) -> RuleTm {
        let BasicRuleData { interface, lhs, rhs } = self.rules.get(&name).unwrap();
        let vars = interface.tm.collect_vars().unwrap();
        let mut subst = zip_eq(vars, terms.iter().cloned()).collect_vec();
        RuleTm {
            rule: PatTm::Res(name, MorTm::List(terms)),
            lhs: lhs.subst(&mut subst),
            rhs: rhs.subst(&mut subst),
        }
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.signature.fmt(f)?;
        writeln!(f, "#/ agents:")?;
        for (name, interface) in &self.agents {
            let ObTmJudgment { tm, ty } = interface;
            let body = RcDoc::text(name.as_str()).append(RcDoc::space()).append(tm.to_doc());
            render_doc(judgment_doc(tm.to_doc(), ty.to_doc(), body), f)?;
            writeln!(f)?;
        }
        writeln!(f, "#/ rules:")?;
        for (&name, BasicRuleData { interface, lhs, rhs }) in &self.rules {
            let ObTmJudgment { tm, ty } = interface;
            let body = RcDoc::text(name.as_str()).append(RcDoc::space()).append(tm.to_doc());
            let rule = mor_doc(body, lhs.to_doc(), rhs.to_doc());
            render_doc(judgment_doc(tm.to_doc(), ty.to_doc(), rule), f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

/// A toy example of a rule-based model (variant 1).
#[cfg(test)]
pub(crate) fn toy_model_v1() -> Model {
    let decls = toy_model_decls("Site", "Site");
    Model::parse(toy_signature_v1(), decls).unwrap()
}

/// A toy example of a rule-based model (variant 2).
#[cfg(test)]
pub(crate) fn toy_model_v2() -> Model {
    let decls = toy_model_decls("SiteA", "SiteB");
    Model::parse(toy_signature_v2(), decls).unwrap()
}

/// A toy example of a rule-based model (single agent).
#[cfg(test)]
pub(crate) fn toy_model_single_agent() -> Model {
    let decls = model_decls_single_agent();
    Model::parse(toy_signature_single_agent(), decls).unwrap()
}

/// A toy example of a rule-based model (emergent agent (dimerization of A and B creates C-binding ability)).
#[cfg(test)]
pub(crate) fn toy_model_emergent_agent() -> Model {
    let decls = model_decls_emergent_agent();
    Model::parse(toy_signature_emergent_agent(), decls).unwrap()
}

/// A toy example of a rule-based model (emergent agent with directionality).
#[cfg(test)]
pub(crate) fn toy_model_directionality() -> Model {
    let decls = model_decls_directionality();
    Model::parse(toy_signature_directionality(), decls).unwrap()
}

/// A toy example of a rule-based model (phospho tyrosine).
#[cfg(test)]
pub(crate) fn toy_model_phospho_tyrosine() -> Model {
    let decls = model_decls_phospho_tyrosine();
    Model::parse(toy_signature_phospho_tyrosine(), decls).unwrap()
}

#[cfg(test)]
fn toy_model_decls(site_a: &str, site_b: &str) -> [ModelDecl; 5] {
    [
        ModelDecl::agent(
            "A",
            [ObTm::var("r"), ObTm::var("s")],
            [Ty::sort("Res"), Ty::sort(site_a)],
        ),
        ModelDecl::agent("B", [ObTm::var("s")], [Ty::sort(site_b)]),
        ModelDecl::agent("K", [], []),
        ModelDecl::rule(
            "bondAB",
            [ObTm::var("r")],
            [Ty::sort("Res")],
            PatTm::tensor([
                PatTm::res("A", [MorTm::var("r"), MorTm::app("empty", [])]),
                PatTm::res("B", [MorTm::app("empty", [])]),
            ]),
            PatTm::let_(
                [ObTm::var("s1"), ObTm::var("s2")],
                MorTm::app("bond", []),
                PatTm::tensor([
                    PatTm::res("A", [MorTm::var("r"), MorTm::var("s1")]),
                    PatTm::res("B", [MorTm::var("s2")]),
                ]),
            ),
        ),
        ModelDecl::rule(
            "phosphorylate",
            [ObTm::var("s")],
            [Ty::sort(site_a)],
            PatTm::tensor([
                PatTm::res("A", [MorTm::app("unphos", []), MorTm::var("s")]),
                PatTm::res("K", []),
            ]),
            PatTm::tensor([
                PatTm::res("A", [MorTm::app("phos", []), MorTm::var("s")]),
                PatTm::res("K", []),
            ]),
        ),
    ]
}

/// A toy example of a rule-based model (variant 2).
// #[cfg(test)]
// pub(crate) fn toy_model_v3() -> Model {
//     let decls = single_agent_model_decls();
//     Model::parse(toy_signature_v2(), decls).unwrap()
// }

#[cfg(test)]
fn model_decls_single_agent() -> [ModelDecl; 3] {
    let ReA = PatTm::res("M", MorTm::var("iota_A"));
    let ReB = PatTm::res("M", MorTm::var("iota_B"));
    let ReK = PatTm::res("M", MorTm::var("iota_K"));

    let SiteA = PatTm::res("M", MorTm::var("iota_SiteA"));
    let SiteB = PatTm::res("M", MorTm::var("iota_SiteB"));
    let Res = PatTm::res("M", MorTm::var("iota_Res"));

    let A = PatTm::tensor([ReA, SiteB, Res]);
    let B = PatTm::tensor([ReB, SiteA]);
    let K = PatTm::tensor([ReK]);

    // @Evan: The next three lines do a bit what the profunctor would do. Do we need this for now?
    // let A = A.subst(&mut vec![(name("iota_A"), MorTm::app("iota_A", MorTm::app("ground_A", [])))]);
    // let B = B.subst(&mut vec![(name("iota_B"), MorTm::app("iota_B", MorTm::app("ground_B", [])))]);
    // let K = K.subst(&mut vec![(name("iota_K"), MorTm::app("iota_K", MorTm::app("ground_K", [])))]);

    let A_phos =
        A.subst(&mut vec![(name("iota_Res"), MorTm::app("iota_Res", MorTm::app("phos", [])))]);
    let A_unphos =
        A.subst(&mut vec![(name("iota_Res"), MorTm::app("iota_Res", MorTm::app("unphos", [])))]);

    let A_free = A.subst(&mut vec![(
        name("iota_SiteB"),
        MorTm::app("iota_SiteB", MorTm::app("empty", [])),
    )]);
    let B_free = B.subst(&mut vec![(
        name("iota_SiteA"),
        MorTm::app("iota_SiteA", MorTm::app("empty", [])),
    )]);
    let AB = PatTm::tensor([A, B]);
    let AB_complex = AB.subst(&mut vec![
        (name("iota_SiteA"), MorTm::app("iota_SiteA", MorTm::app("s1", []))),
        (name("iota_SiteB"), MorTm::app("iota_SiteB", MorTm::app("s2", []))),
    ]);
    let AB_complex =
        PatTm::let_([ObTm::var("s1"), ObTm::var("s2")], MorTm::app("bond", []), AB_complex); // TODO: ask Evan how to do this
    [
        ModelDecl::agent("M", [ObTm::var("m")], [Ty::sort("ReMonomer")]),
        ModelDecl::rule(
            "bondAB",
            [ObTm::var("r")],
            [Ty::sort("Res")],
            PatTm::tensor([
                A_free.subst(&mut vec![(
                    name("iota_Res"),
                    MorTm::app("iota_Res", MorTm::app("r", [])),
                )]), // @Evan: I believe the substitution here is required to introduce the variable "r"
                B_free,
            ]),
            AB_complex
                .subst(&mut vec![(name("iota_Res"), MorTm::app("iota_Res", MorTm::app("r", [])))]),
        ),
        ModelDecl::rule(
            "phosphorylate",
            [ObTm::var("s")],
            [Ty::sort("SiteB")],
            PatTm::tensor([
                A_unphos.subst(&mut vec![(
                    name("iota_SiteB"),
                    MorTm::app("iota_SiteB", MorTm::app("s", [])),
                )]),
                K.clone(),
            ]), // @Evan: what do you think of the requirement to clone here?
            PatTm::tensor([
                A_phos.subst(&mut vec![(
                    name("iota_SiteB"),
                    MorTm::app("iota_SiteB", MorTm::app("s", [])),
                )]),
                K,
            ]),
        ),
    ]
}

#[cfg(test)]
fn model_decls_emergent_agent() -> [ModelDecl; 5] {
    let AB = PatTm::let_(
        [ObTm::var("ab"), ObTm::var("ba")],
        MorTm::app("bond", []),
        PatTm::tensor([
            PatTm::res("A", [MorTm::var("ab"), MorTm::var("c")]),
            PatTm::res("B", [MorTm::var("ba"), MorTm::var("c")]),
        ]),
    );
    let AC = PatTm::let_(
        [ObTm::var("ac"), ObTm::var("ca")],
        MorTm::app("bond", []),
        PatTm::tensor([
            PatTm::res("A", [MorTm::var("b"), MorTm::var("ac")]),
            PatTm::res("C", [MorTm::var("ca"), MorTm::var("cb")]),
        ]),
    );
    let ABC_incomplete = PatTm::let_(
        [ObTm::var("ac"), ObTm::var("ca")],
        MorTm::app("bond", []),
        PatTm::tensor([AB, PatTm::res("C", [MorTm::var("ca"), MorTm::var("cb")])]),
    );
    let ABC =
        PatTm::let_([ObTm::var("bc"), ObTm::var("cb")], MorTm::app("bond", []), ABC_incomplete);
    // let BC = PatTm::let_(
    //             [ObTm::var("bc1"), ObTm::var("bc2")],
    //             MorTm::app("bond", []),
    //             PatTm::tensor([
    //                 PatTm::res("B", [MorTm::var("ab2"), MorTm::var("bc1")]),
    //                 PatTm::res("C", [MorTm::var("ac2"), MorTm::var("bc2")])
    //             ]
    //             ));
    [
        ModelDecl::agent(
            "A",
            [ObTm::var("ab"), ObTm::var("ac")],
            [Ty::sort("SiteB"), Ty::sort("SiteC")],
        ),
        ModelDecl::agent(
            "B",
            [ObTm::var("ba"), ObTm::var("bc")],
            [Ty::sort("SiteA"), Ty::sort("SiteC")],
        ),
        ModelDecl::agent(
            "C",
            [ObTm::var("ca"), ObTm::var("cb")],
            [Ty::sort("SiteAB"), Ty::sort("SiteAB")],
        ),
        ModelDecl::rule(
            "R_dimerization",
            [ObTm::var("ca"), ObTm::var("cb")],
            [Ty::sort("SiteC"), Ty::sort("SiteC")],
            PatTm::tensor([
                PatTm::res("A", [MorTm::var("e_B")]),
                PatTm::res("B", [MorTm::var("e_A")]),
            ]),
            PatTm::let_(
                [ObTm::var("s1"), ObTm::var("s2")], // TODO: harmonize variable naming convention
                MorTm::app("bond", []),
                PatTm::tensor([
                    PatTm::res("A", [MorTm::var("s1"), MorTm::var("e_C")]),
                    PatTm::res("B", [MorTm::var("s2"), MorTm::var("e_C")]),
                ]),
            ),
        ),
        ModelDecl::rule(
            "R_trimerization",
            [],
            [],
            PatTm::tensor([
                PatTm::let_(
                    [ObTm::var("s1"), ObTm::var("s2")],
                    MorTm::app("bond", []),
                    PatTm::tensor([
                        PatTm::res("A", [MorTm::var("s1"), MorTm::var("e_C")]),
                        PatTm::res("B", [MorTm::var("s2"), MorTm::var("e_C")]),
                    ]),
                ),
                PatTm::res("C", [MorTm::var("e_AB"), MorTm::var("e_AB")]),
            ]),
            ABC,
        ),
    ]
}

#[cfg(test)]
fn model_decls_directionality() -> [ModelDecl; 5] {
    // let AB = PatTm::let_(
    //     [ObTm::var("ab"), ObTm::var("ba")],
    //     MorTm::app("bond", []),
    //     PatTm::tensor([
    //         PatTm::res("A", [MorTm::var("id_head"), MorTm::var("id_Site_C1"), MorTm::var("ab")]),
    //         PatTm::res("B", [MorTm::var("ba"), MorTm::var("id_Site_C2"), MorTm::var("id_tail")]),
    //     ]),
    // );
    // let AC = PatTm::let_(
    //     [ObTm::var("ac"), ObTm::var("ca")],
    //     MorTm::app("bond", []),
    //     PatTm::tensor([
    //         PatTm::res("A", [MorTm::var("b"), MorTm::var("ac")]),
    //         PatTm::res("C", [MorTm::var("ca"), MorTm::var("cb")]),
    //     ]),
    // );
    // let ABC_incomplete = PatTm::let_(
    //     [ObTm::var("ac"), ObTm::var("ca")],
    //     MorTm::app("bond", []),
    //     PatTm::tensor([AB, PatTm::res("C", [MorTm::var("ca"), MorTm::var("cb")])]),
    // );
    // let ABC =
    //     PatTm::let_([ObTm::var("bc"), ObTm::var("cb")], MorTm::app("bond", []), ABC_incomplete);
    // let BC = PatTm::let_(
    //             [ObTm::var("bc1"), ObTm::var("bc2")],
    //             MorTm::app("bond", []),
    //             PatTm::tensor([
    //                 PatTm::res("B", [MorTm::var("ab2"), MorTm::var("bc1")]),
    //                 PatTm::res("C", [MorTm::var("ac2"), MorTm::var("bc2")])
    //             ]
    //             ));
    let ABC = PatTm::let_(
        [ObTm::var("ac"), ObTm::var("ca")],
        MorTm::app("bond_Ch", []),
        PatTm::let_(
            [ObTm::var("bc"), ObTm::var("cb")],
            MorTm::app("bond_Ct", []),
            PatTm::let_(
                [ObTm::var("ab"), ObTm::var("ba")],
                MorTm::app("bond_AB", []),
                PatTm::tensor([
                    PatTm::res("A", [MorTm::var("id_head"), MorTm::var("ac"), MorTm::var("ab")]),
                    PatTm::res("B", [MorTm::var("ba"), MorTm::var("bc"), MorTm::var("id_tail")]),
                    PatTm::res("C", [MorTm::var("ca"), MorTm::var("cb")]),
                ]),
            ),
        ),
    );
    [
        ModelDecl::agent(
            "A",
            [ObTm::var("ah"), ObTm::var("ac"), ObTm::var("at")],
            [Ty::sort("head"), Ty::sort("Site_C"), Ty::sort("tail")],
        ),
        ModelDecl::agent(
            "B",
            [ObTm::var("bh"), ObTm::var("bc"), ObTm::var("bt")],
            [Ty::sort("head"), Ty::sort("Site_C"), Ty::sort("tail")],
        ),
        ModelDecl::agent(
            "C",
            [ObTm::var("ch"), ObTm::var("ct")],
            [Ty::sort("Site_ABh"), Ty::sort("Site_ABt")],
        ),
        ModelDecl::rule(
            "R_dimerization",
            [
                ObTm::var("id_head"),
                ObTm::var("id_Site_C1"),
                ObTm::var("id_Site_C2"),
                ObTm::var("id_tail"),
            ],
            [Ty::sort("head"), Ty::sort("Site_C"), Ty::sort("Site_C"), Ty::sort("tail")],
            PatTm::tensor([
                PatTm::res(
                    "A",
                    [MorTm::var("id_head"), MorTm::var("id_Site_C1"), MorTm::var("e_t")],
                ),
                PatTm::res(
                    "B",
                    [MorTm::var("e_h"), MorTm::var("id_Site_C2"), MorTm::var("id_tail")],
                ),
            ]),
            PatTm::let_(
                [ObTm::var("s1"), ObTm::var("s2")], // TODO: harmonize variable naming convention
                MorTm::app("bond_AB", []),
                PatTm::tensor([
                    PatTm::res(
                        "A",
                        [MorTm::var("id_head"), MorTm::var("id_Site_C1"), MorTm::var("s1")],
                    ),
                    PatTm::res(
                        "B",
                        [MorTm::var("s2"), MorTm::var("id_Site_C2"), MorTm::var("id_tail")],
                    ),
                ]),
            ),
        ),
        ModelDecl::rule(
            "R_trimerization",
            [ObTm::var("id_head"), ObTm::var("id_tail")],
            [Ty::sort("head"), Ty::sort("tail")],
            PatTm::tensor([
                PatTm::let_(
                    [ObTm::var("s1"), ObTm::var("s2")],
                    MorTm::app("bond", []),
                    PatTm::tensor([
                        PatTm::res(
                            "A",
                            [MorTm::var("id_head"), MorTm::var("e_C"), MorTm::var("s1")],
                        ),
                        PatTm::res(
                            "B",
                            [MorTm::var("s2"), MorTm::var("e_C"), MorTm::var("id_tail")],
                        ),
                    ]),
                ),
                PatTm::res("C", [MorTm::var("e_Ch"), MorTm::var("e_Ct")]),
            ]),
            ABC,
        ),
    ]
}

#[cfg(test)]
fn model_decls_phospho_tyrosine() -> [ModelDecl; 4] {
    [
        ModelDecl::agent("A", [ObTm::var("x")], [Ty::sort("SH2")]),
        ModelDecl::agent("C", [ObTm::var("y")], [Ty::sort("Tyr")]),
        ModelDecl::rule(
            "R_phosphorylation",
            [],
            [],
            PatTm::res("A", [MorTm::app("u", MorTm::app("e_xtyr", []))]),
            PatTm::res("A", [MorTm::app("p", MorTm::app("e_xtyr", []))]),
        ),
        ModelDecl::rule(
            "R_dimerization",
            [],
            [],
            PatTm::tensor([
                PatTm::res("A", [MorTm::var("e_sh2")]),
                PatTm::res("C", [MorTm::app("p", MorTm::app("e_xtyr", []))]),
            ]),
            PatTm::let_(
                [ObTm::var("s1"), ObTm::var("s2")],
                MorTm::app("bond", []),
                PatTm::tensor([
                    PatTm::res("A", [MorTm::var("s1")]),
                    PatTm::res("C", [MorTm::app("p", [MorTm::var("s2")])]),
                ]),
            ),
        ),
    ]
} // TODO: Implement this model as preorder (currently `u` and `p` are parallel morphisms)

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn parse() {
        let expected = expect![[r#"
            #/ sorts:
            Res
            Site
            #/ operations:
            unphos : [] → Res
            phos : [] → Res
            empty : [] → Site
            bond : [] → ⊗ [Site, Site]
            #/ agents:
            [r, s] : [Res, Site] ⊢ A [r, s]
            [s] : [Site] ⊢ B [s]
            [] : [] ⊢ K []
            #/ rules:
            [r] : [Res] ⊢
              bondAB [r]
                : (A [r, empty []], B [empty []])
                → let [s1, s2] = bond [] in (A [r, s1], B [s2])
            [s] : [Site] ⊢
              phosphorylate [s] : (A [unphos [], s], K []) → (A [phos [], s], K [])
        "#]];
        expected.assert_eq(&toy_model_v1().to_string());

        let expected = expect![[r#"
            #/ sorts:
            Res
            SiteA
            SiteB
            #/ operations:
            unphos : [] → Res
            phos : [] → Res
            emptyA : [] → SiteA
            emptyB : [] → SiteB
            bond : [] → ⊗ [SiteA, SiteB]
            #/ agents:
            [r, s] : [Res, SiteA] ⊢ A [r, s]
            [s] : [SiteB] ⊢ B [s]
            [] : [] ⊢ K []
            #/ rules:
            [r] : [Res] ⊢
              bondAB [r]
                : (A [r, empty []], B [empty []])
                → let [s1, s2] = bond [] in (A [r, s1], B [s2])
            [s] : [SiteA] ⊢
              phosphorylate [s] : (A [unphos [], s], K []) → (A [phos [], s], K [])
        "#]];
        expected.assert_eq(&toy_model_v2().to_string());

        let expected = expect![[r#"
            #/ sorts:
            ReMonomer
            ReA
            ReB
            ReK
            SiteA
            SiteB
            Res
            #/ operations:
            iota_A : [ReA] → ReMonomer
            iota_B : [ReB] → ReMonomer
            iota_K : [ReK] → ReMonomer
            iota_SiteA : [SiteA] → ReMonomer
            iota_SiteB : [SiteB] → ReMonomer
            iota_Res : [Res] → ReMonomer
            phos : [] → Res
            unphos : [] → Res
            ground_A : [] → ReA
            ground_B : [] → ReB
            ground_K : [] → ReK
            emptyA : [] → SiteA
            emptyB : [] → SiteB
            bond : [] → ⊗ [SiteA, SiteB]
            #/ agents:
            [m] : [ReMonomer] ⊢ M [m]
            #/ rules:
            [r] : [Res] ⊢
              bondAB [r]
                : (
                  (M iota_A, M iota_SiteB empty [], M iota_Res r []),
                  (M iota_B, M iota_SiteA empty [])
                )
                → let [s1, s2] = bond [] in
                  (
                    (M iota_A, M iota_SiteB s2 [], M iota_Res r []),
                    (M iota_B, M iota_SiteA s1 [])
                  )
            [s] : [SiteB] ⊢
              phosphorylate [s]
                : ((M iota_A, M iota_SiteB s [], M iota_Res unphos []), (M iota_K))
                → ((M iota_A, M iota_SiteB s [], M iota_Res phos []), (M iota_K))
        "#]];
        expected.assert_eq(&toy_model_single_agent().to_string());

        let expected = expect![[r#"
            #/ sorts:
            SiteA
            SiteB
            SiteC
            SiteAB
            #/ operations:
            e_A : [] → SiteA
            e_B : [] → SiteB
            e_C : [] → SiteC
            e_AB : [] → SiteAB
            bond_AB : [] → ⊗ [SiteA, SiteB]
            bond_C : [] → ⊗ [SiteAB, SiteC]
            #/ agents:
            [ab, ac] : [SiteB, SiteC] ⊢ A [ab, ac]
            [ba, bc] : [SiteA, SiteC] ⊢ B [ba, bc]
            [ca, cb] : [SiteAB, SiteAB] ⊢ C [ca, cb]
            #/ rules:
            [ca, cb] : [SiteC, SiteC] ⊢
              R_dimerization [ca, cb]
                : (A [e_B], B [e_A])
                → let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C])
            [] : [] ⊢
              R_trimerization []
                : (let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]), C [e_AB, e_AB])
                → let [bc, cb] = bond [] in
                  let [ac, ca] = bond [] in
                    (let [ab, ba] = bond [] in (A [ab, c], B [ba, c]), C [ca, cb])
        "#]]; // TODO: fix unbound variables in nested let statement
        expected.assert_eq(&toy_model_emergent_agent().to_string());

        let expected = expect![[r#"
            #/ sorts:
            head
            tail
            Site_C
            Site_ABh
            Site_ABt
            #/ operations:
            e_h : [] → head
            e_t : [] → tail
            e_C : [] → Site_C
            e_ABh : [] → Site_ABh
            e_ABt : [] → Site_ABt
            bond_AB : [] → ⊗ [head, tail]
            bond_Ch : [] → ⊗ [Site_ABh, Site_C]
            bond_Ct : [] → ⊗ [Site_ABt, Site_C]
            #/ agents:
            [ah, ac, at] : [head, Site_C, tail] ⊢ A [ah, ac, at]
            [bh, bc, bt] : [head, Site_C, tail] ⊢ B [bh, bc, bt]
            [ch, ct] : [Site_ABh, Site_ABt] ⊢ C [ch, ct]
            #/ rules:
            [id_head, id_Site_C1, id_Site_C2, id_tail] : [head, Site_C, Site_C, tail] ⊢
              R_dimerization [id_head, id_Site_C1, id_Site_C2, id_tail]
                : (A [id_head, id_Site_C1, e_t], B [e_h, id_Site_C2, id_tail])
                → let [s1, s2] = bond_AB [] in
                  (A [id_head, id_Site_C1, s1], B [s2, id_Site_C2, id_tail])
            [id_head, id_tail] : [head, tail] ⊢
              R_trimerization [id_head, id_tail]
                : (
                  let [s1, s2] = bond [] in (A [id_head, e_C, s1], B [s2, e_C, id_tail]),
                  C [e_Ch, e_Ct]
                )
                → let [ac, ca] = bond_Ch [] in
                  let [bc, cb] = bond_Ct [] in
                    let [ab, ba] = bond_AB [] in
                      (A [id_head, ac, ab], B [ba, bc, id_tail], C [ca, cb])
        "#]]; // TODO: fix unbound variables in nested let statement
        expected.assert_eq(&toy_model_directionality().to_string());

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
            [] : [] ⊢ R_phosphorylation [] : A [u e_xtyr []] → A [p e_xtyr []]
            [] : [] ⊢
              R_dimerization []
                : (A [e_sh2], C [p e_xtyr []])
                → let [s1, s2] = bond [] in (A [s1], C [p [s2]])
        "#]];
        expected.assert_eq(&toy_model_phospho_tyrosine().to_string());
    }
}
