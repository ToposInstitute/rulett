//! Rule-based models.

use std::{fmt, rc::Rc};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use super::{gensym::*, prelude::*, theory::*, tm::*, ty::*};

/// Pattern term in a rule-based model.
///
/// A pattern is represented as a restriction of a list of agents along a
/// morphism term.
pub struct PatternTm {
    pub agents: Vec<Name>,
    pub tm: MorTm,
}

impl fmt::Display for PatternTm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.agents.iter().join(", "), self.tm)
    }
}

/// Declaration in the definition of a rule-based model.
pub enum ModelDecl {
    /// Declaration of an agent.
    ///
    /// The variable names defined by the [`ObTm`] are logically superfluous,
    /// but are included as a kind of documentation and for consistency with
    /// rule declarations, where they are necessary.
    Agent { name: Name, interface: (ObTm, Ty) },

    /// Declaration of a basic rule.
    Rule {
        name: Name,
        interface: (ObTm, Ty),
        lhs: PatternTm,
        rhs: PatternTm,
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
            ModelDecl::Rule { .. } => todo!("parsing rules"),
        }
    }

    /// Adds an agent with the given name and interface to the model.
    pub fn add_agent(&mut self, name: Name, tm: ObTm, ty: Ty) -> Result<(), String> {
        if !self
            .signature
            .check_ty(&ty, &Kind::list(Kind::prim()))
            .map_err(|err| format!("interface has invalid type: {err}"))?
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
            let (tm, ty) = (&interface.tm, &interface.ty);
            writeln!(f, "{tm} : {ty} ⊢ {name} {tm}")?;
        }
        Ok(())
    }
}

impl Model {
    /// Derives all species in the model.
    ///
    /// A *closed pattern* is a pattern with trivial interface. A *species* is
    /// an indecomposable closed pattern, i.e., a closed pattern that cannot be
    /// expressed as a non-trivial product of other closed patterns.
    pub fn species(&self, max_agents: usize) -> Vec<PatternTm> {
        let agents_in_signature = || self.agents.keys().copied();
        (1..=max_agents)
            .flat_map(|n| {
                agents_in_signature().combinations_with_replacement(n).flat_map(|agents| {
                    println!("{:?}", agents);
                    self.species_from(&agents)
                        .into_iter()
                        .map(move |tm| PatternTm { agents: agents.clone(), tm })
                })
            })
            .collect()
    }

    /// Derives species by restricting the given list of agents.
    fn species_from(&self, agents: &[Name]) -> Vec<MorTm> {
        // Initialize search state.
        let mut interface = Vec::new();
        for (i, agent) in agents.iter().enumerate() {
            let agent_interface = self.agents.get(agent).unwrap().collect_typed_vars();
            interface.extend(agent_interface.into_iter().map(|(name, sort)| IntermediateVar {
                name,
                sort,
                component: i,
            }));
        }
        println!("{:?}", interface);
        let tm = MorTm::list(interface.iter().map(|var| MorTm::var(var.name)));
        let components = Rc::new(QuickUnionUf::new(agents.len()));
        let state = SpeciesState { tm, interface, components };
        // Run the search.
        let finder = SpeciesFinder::new(&self.signature);
        let mut results = Vec::new();
        finder.find(state, &mut results);
        results
    }
}

struct SpeciesFinder<'a> {
    signature: &'a Signature,
    /// Index from flattened operation codomains to operations.
    cod_index: HashMap<Vec<Name>, Vec<Name>>,
}

#[derive(Clone)]
struct SpeciesState {
    tm: MorTm,
    interface: Vec<IntermediateVar>,
    components: Rc<QuickUnionUf<UnionBySize>>,
}

#[derive(Clone, Copy, Debug)]
struct IntermediateVar {
    name: Name,
    sort: Name,
    component: usize,
}

impl IntermediateVar {
    fn replace_component(self, component: usize) -> Self {
        Self {
            name: self.name,
            sort: self.sort,
            component,
        }
    }
}

impl<'a> SpeciesFinder<'a> {
    fn new(signature: &'a Signature) -> Self {
        let mut cod_index = HashMap::<_, Vec<_>>::new();
        for (name, _, cod) in signature.operations() {
            cod_index.entry(cod.collect_sorts()).or_default().push(name);
        }
        Self { signature, cod_index }
    }

    fn find(&self, state: SpeciesState, results: &mut Vec<MorTm>) {
        let SpeciesState { interface, tm, components: uf } = state;

        // Success condition: found a closed term.
        if interface.is_empty() {
            results.push(tm);
            return;
        }

        for idxs in (0..interface.len()).powerset() {
            // Don't restrict along co-nullary operations as that causes
            // infinite blow-up. Such operations, which include
            // [scalars](https://ncatlab.org/nlab/show/monoidal+category#scalars),
            // also seem pointless, but perhaps they're good for something?
            if idxs.is_empty() {
                continue;
            }

            // Get co-applicable operations, bailing early if there are none.
            let sorts = idxs.iter().map(|i| interface[*i].sort).collect_vec();
            println!("{:?} {:?}", idxs, sorts);
            let Some(operations) = self.cod_index.get(&sorts).filter(|ops| !ops.is_empty()) else {
                continue;
            };

            // Union components involved in restricting along these indices.
            let mut uf = uf.clone();
            let mut has_merged = false;
            let mut components = idxs.iter().map(|i| interface[*i].component);
            let first = components.next().unwrap();
            for component in components {
                if Rc::make_mut(&mut uf).union(first, component) {
                    has_merged = true;
                }
            }
            let component = Rc::make_mut(&mut uf).find(first);

            // Construct interface that remains after restricting along indices.
            let interface_kept = interface
                .iter()
                .enumerate()
                .filter_map(|(i, &var)| {
                    if idxs.contains(&i) {
                        return None;
                    }
                    if has_merged {
                        Some(var.replace_component(Rc::make_mut(&mut uf).find(var.component)))
                    } else {
                        Some(var)
                    }
                })
                .collect_vec();

            // Restrict along each co-applicable operation and recurse.
            for op in operations {
                let (dom, cod) = self.signature.interface(op).unwrap();

                let interface_added = dom
                    .collect_sorts()
                    .into_iter()
                    .map(|sort| {
                        let name = gen_var_with_sort(&sort);
                        IntermediateVar { name, sort, component }
                    })
                    .collect_vec();
                let args = MorTm::list(interface_added.iter().map(|var| MorTm::var(var.name)));
                let app = MorTm::app(*op, args);

                println!("{op}: kept {:?}, added {:?}", interface_kept, interface_added);

                let tm = if matches!(cod, Ty::Sort(_)) {
                    let i = idxs.iter().exactly_one().unwrap();
                    let var = interface[*i].name;
                    tm.subst(&mut vec![(var, app)])
                } else {
                    // TODO: Let binding.
                    continue;
                };

                let mut interface = interface_added;
                interface.extend(interface_kept.iter());
                self.find(SpeciesState { tm, interface, components: uf.clone() }, results)
            }
        }
    }
}

fn gen_var_with_sort(sort: &Name) -> Name {
    gensym(&sort.to_lowercase())
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

    #[test]
    fn species() {
        let model = toy_model();
        let expected = expect![[""]];
        expected.assert_eq(&model.species(1).into_iter().join("\n"));
    }
}
