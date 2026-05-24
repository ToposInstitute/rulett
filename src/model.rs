//! Rule-based models.

use std::fmt;

use super::{prelude::*, theory::*, tm::*, ty::*};

/// A declaration in the definition of a rule-based model.
pub enum ModelDecl {
    /// Declaration of an agent.
    Agent { name: Name, interface: (ObTm, Ty) },
    /// Declaration of a basic rule.
    Rule {
        name: Name,
        interface: (ObTm, Ty),
        lhs: Pattern,
        rhs: Pattern,
    },
}

impl ModelDecl {
    /// Smart constructor for [`Agent`](Self::Agent) variant.
    pub fn agent(name: impl Into<Name>, tm: ObTm, ty: Ty) -> Self {
        Self::Agent { name: name.into(), interface: (tm, ty) }
    }
}

/// A rule-based model.
pub struct Model {
    signature: Signature,
    agents: IndexMap<Name, ObTmJudgment>,
    // TODO: Rules
}

impl Model {
    /// Constructs an empty model over a signature.
    pub fn new(signature: Signature) -> Self {
        Self { signature, agents: Default::default() }
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
            ModelDecl::Rule { .. } => Ok(()), // TODO
        }
    }

    /// Adds an agent with the given name and interface to the model.
    pub fn add_agent(&mut self, name: Name, tm: ObTm, ty: Ty) -> Result<(), String> {
        if !self
            .signature
            .check_ty(&ty, &Kind::list(Kind::prim()))
            .map_err(|err| format!("invalid interface type: {err}"))?
        {
            return Err(format!("interface type should be a list of sorts, received: {ty}"));
        }
        let judgment =
            ObTmJudgment::judge(tm, ty).map_err(|err| format!("invalid interface: {err}"))?;
        if self.agents.insert(name, judgment).is_some() {
            return Err(format!("{name} already defined"));
        }
        Ok(())
    }

    /// Iterates over the agents in the model.
    pub fn agents(&self) -> impl Iterator<Item = (Name, &ObTmJudgment)> {
        self.agents.iter().map(|(name, judgment)| (*name, judgment))
    }
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.signature.fmt(f)?;
        writeln!(f, "#/ agents:")?;
        for (name, interface) in self.agents() {
            let (tm, ty) = (interface.tm(), interface.ty());
            writeln!(f, "{tm} : {ty} ⊢ {name} {tm}")?;
        }
        Ok(())
    }
}

/// Pattern in a rule-based model.
///
/// A pattern is represented as a restriction of a list of agents along a term.
pub struct Pattern {
    pub agents: Vec<Name>,
    pub term: MorTm,
}

/// Derived rule in a rule-based model.
pub struct Rule {
    pub rules: Vec<Name>,
    pub term: MorTm,
}

/// Our favorite toy example of a ruled-based model.
#[cfg(test)]
pub(crate) fn toy_model() -> Model {
    let decls = [
        ModelDecl::agent(
            "A",
            ObTm::list([ObTm::var("r"), ObTm::var("s")]),
            Ty::list([Ty::sort("Res"), Ty::sort("Site")]),
        ),
        ModelDecl::agent("B", ObTm::list([ObTm::var("s")]), Ty::list([Ty::sort("Site")])),
        ModelDecl::agent("K", ObTm::list([]), Ty::list([])),
    ];
    Model::parse(toy_signature(), decls).unwrap()
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
        "#]];
        expected.assert_eq(&toy_model().to_string());
    }
}
