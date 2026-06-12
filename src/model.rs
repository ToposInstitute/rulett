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
        let vars = interface.tm.vars().unwrap();
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

/// A toy example of a ruled-based model (variant 1).
#[cfg(test)]
pub(crate) fn toy_model_v1() -> Model {
    let decls = toy_model_decls("Site", "Site", "empty", "empty");
    Model::parse(toy_signature_v1(), decls).unwrap()
}

/// A toy example of a ruled-based model (variant 2).
#[cfg(test)]
pub(crate) fn toy_model_v2() -> Model {
    let decls = toy_model_decls("SiteA", "SiteB", "emptyA", "emptyB");
    Model::parse(toy_signature_v2(), decls).unwrap()
}

#[cfg(test)]
fn toy_model_decls(site_a: &str, site_b: &str, empty_a: &str, empty_b: &str) -> [ModelDecl; 5] {
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
                PatTm::res("A", [MorTm::var("r"), MorTm::app(empty_a, [])]),
                PatTm::res("B", [MorTm::app(empty_b, [])]),
            ]),
            PatTm::let_(
                ObTm::tensor([ObTm::var("s1"), ObTm::var("s2")]),
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
                → let (s1, s2) = bond [] in (A [r, s1], B [s2])
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
                : (A [r, emptyA []], B [emptyB []])
                → let (s1, s2) = bond [] in (A [r, s1], B [s2])
            [s] : [SiteA] ⊢
              phosphorylate [s] : (A [unphos [], s], K []) → (A [phos [], s], K [])
        "#]];
        expected.assert_eq(&toy_model_v2().to_string());
    }
}
