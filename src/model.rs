//! Rule-based models.

use std::{fmt, rc::Rc};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use super::{prelude::*, theory::*, tm::*, ty::*};

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
        lhs: PatternTm,
        rhs: PatternTm,
    ) -> Self {
        Self::Rule {
            name: name.into(),
            interface: (tm.into(), ty.into()),
            lhs,
            rhs,
        }
    }
}

struct BasicRuleData {
    interface: ObTmJudgment,
    lhs: PatternTm,
    rhs: PatternTm,
}

/// A rule-based model.
pub struct Model {
    signature: Signature,
    agents: IndexMap<Name, ObTmJudgment>,
    rules: IndexMap<Name, BasicRuleData>,
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
        if self.agents.insert(name, interface).is_some() {
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
        lhs: PatternTm,
        rhs: PatternTm,
    ) -> Result<(), String> {
        let interface = self.check_interface(tm, ty)?;
        // TODO: Type check left- and right-hand sides!
        let data = BasicRuleData { interface, lhs, rhs };
        if self.rules.insert(name, data).is_some() {
            return Err(format!("{name} already defined"));
        }
        Ok(())
    }

    /// Checks that interface to agent or rule is well-typed.
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
            let ObTmJudgment { tm, ty } = interface;
            writeln!(f, "{tm} : {ty} ⊢ {name} {tm}")?;
        }
        writeln!(f, "#/ rules:")?;
        for (&name, BasicRuleData { interface, lhs, rhs }) in &self.rules {
            let ObTmJudgment { tm, ty } = interface;
            writeln!(f, "{tm} : {ty} ⊢ {name} {tm} : {lhs} → {rhs}")?;
        }
        Ok(())
    }
}

impl Model {
    /// Derives all species generated by the model.
    ///
    /// A *closed pattern* is a pattern with trivial interface. A *species* is
    /// an indecomposable closed pattern, i.e., a closed pattern that cannot be
    /// expressed as a non-trivial product of other closed patterns.
    pub fn species(&self, max_agents: usize) -> Vec<PatternTm> {
        let finder = SpeciesFinder::new(&self.signature);
        (1..=max_agents)
            .flat_map(|n| self.agents.keys().copied().combinations_with_replacement(n))
            .flat_map(|agents| {
                let agents_with_interfaces = agents
                    .into_iter()
                    .map(|agent| (agent, self.agents.get(&agent).unwrap().collect_typed_vars()));
                finder.find(agents_with_interfaces)
            })
            .collect()
    }
}

struct SpeciesFinder<'a> {
    signature: &'a Signature,
    /// Index from flattened operation codomains to operations.
    cod_index: HashMap<Vec<Name>, Vec<Name>>,
}

#[derive(Clone)]
struct SpeciesState {
    tm: PatternTm,
    interface: Vec<IntermediateVar>,
    // Wrap the union find in `Rc` since we don't have to mutate it at each
    // branch point, only when restricting along an operation of co-arity >= 2.
    uf: Rc<QuickUnionUf<UnionBySize>>,
    min_match_idx: usize,
}

#[derive(Clone, Copy, Debug)]
struct IntermediateVar {
    name: Name,
    sort: Name,
    component: usize,
}

impl<'a> SpeciesFinder<'a> {
    fn new(signature: &'a Signature) -> Self {
        let mut cod_index = HashMap::<_, Vec<_>>::new();
        for (name, _, cod) in signature.operations() {
            cod_index.entry(cod.collect_sorts()).or_default().push(name);
        }
        Self { signature, cod_index }
    }

    fn find(
        &self,
        interfaces: impl IntoIterator<Item = (Name, IndexMap<Name, Name>)>,
    ) -> Vec<PatternTm> {
        // Collect all variables, then ensure they are unique.
        let mut agents: Vec<(Name, Vec<Name>)> = Vec::new();
        let mut variables = Vec::new();
        let mut interfaces = interfaces.into_iter().enumerate();
        while let Some((i, (agent, interface))) = interfaces.next() {
            // Degenerate case: if any agent's interface is empty, exit early
            // since any non-trivial product with the agent is decomposable.
            if interface.is_empty() {
                return if i == 0 && interfaces.next().is_none() {
                    vec![PatternTm::res(agent, MorTm::list([]))]
                } else {
                    vec![]
                };
            }

            let (vars, sorts): (Vec<_>, Vec<_>) = interface.into_iter().unzip();
            agents.push((agent, sorts));
            variables.extend(vars);
        }
        uniquify_names(&mut variables);

        // Build initial term.
        let mut terms = Vec::new();
        let mut interface = Vec::new();
        let mut variables = variables.into_iter();
        for (i, (agent, sorts)) in agents.into_iter().enumerate() {
            let mut vars = Vec::new();
            for sort in sorts {
                let var = variables.next().unwrap();
                interface.push(IntermediateVar { name: var, sort, component: i });
                vars.push(MorTm::var(var));
            }
            terms.push(PatternTm::res(agent, MorTm::list(vars)));
        }
        let n = terms.len();
        let tm = if n == 1 {
            terms.remove(0)
        } else {
            PatternTm::tensor(PatternTm::list(terms))
        };

        // Initialize the search state, then run the search.
        let uf = Rc::new(QuickUnionUf::new(n));
        let state = SpeciesState { tm, interface, uf, min_match_idx: 0 };
        let mut results = Vec::new();
        self.recurse(state, &mut results);
        results
    }

    fn recurse(&self, state: SpeciesState, results: &mut Vec<PatternTm>) {
        let SpeciesState { interface, tm, uf, min_match_idx } = state;

        // Success condition: found a closed term.
        if interface.is_empty() {
            results.push(tm);
            return;
        }

        for idxs in (0..interface.len()).powerset() {
            // To avoid duplicate species, skip any subsets that do not include
            // at least one index beyond the current minimum.
            //
            // As a special case, never restrict along co-nullary operations as
            // that causes infinite blow-up. Such operations, which include
            // [scalars](https://ncatlab.org/nlab/show/monoidal+category#scalars),
            // also seem pointless, but perhaps they're good for something?
            let min_match_idx = match idxs.iter().min() {
                Some(&n) if n >= min_match_idx => n,
                _ => {
                    continue;
                }
            };

            // Get co-applicable operations, bailing early if there are none.
            let sorts = idxs.iter().map(|i| interface[*i].sort).collect_vec();
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
            let component = if has_merged {
                Rc::make_mut(&mut uf).find(first)
            } else {
                first
            };

            // Construct interface that remains after restricting along indices.
            let interface_kept = interface
                .iter()
                .enumerate()
                .filter_map(|(i, &(mut var))| {
                    if idxs.contains(&i) {
                        return None;
                    }
                    if has_merged {
                        var.component = Rc::make_mut(&mut uf).find(var.component);
                    }
                    Some(var)
                })
                .collect_vec();

            // Restrict along each co-applicable operation and recurse.
            for op in operations {
                let (dom, cod) = self.signature.interface(op).unwrap();

                let mut interface_added = dom
                    .collect_sorts()
                    .into_iter()
                    .map(|sort| {
                        let name = gen_var_with_sort(&sort);
                        IntermediateVar { name, sort, component }
                    })
                    .collect_vec();

                // If this component is being closed off but does not overlap
                // with the (non-trivial) remaining interface, then any further
                // pattern derived will be decomposable, so skip it.
                if interface_added.is_empty()
                    && !interface_kept.is_empty()
                    && interface_kept.iter().all(|var| var.component != component)
                {
                    continue;
                }

                let restrict_at = if matches!(cod, Ty::Sort(_)) {
                    let i = idxs.iter().exactly_one().unwrap();
                    ObTm::var(interface[*i].name)
                } else {
                    let vars = idxs.iter().map(|i| ObTm::var(interface[*i].name));
                    ObTm::tensor(ObTm::list(vars))
                };

                let args = MorTm::list(interface_added.iter().map(|var| MorTm::var(var.name)));
                let restrict_along = MorTm::app(*op, args);

                let mut interface = interface_kept.clone();
                interface.append(&mut interface_added);
                let state = SpeciesState {
                    tm: tm.restrict(restrict_at, restrict_along),
                    interface,
                    uf: uf.clone(),
                    min_match_idx,
                };
                self.recurse(state, results)
            }
        }
    }
}

fn gen_var_with_sort(sort: &Name) -> Name {
    gensym(&sort.to_lowercase())
}

/// A toy example of a ruled-based model (variant 1).
#[cfg(test)]
pub(crate) fn toy_model_v1() -> Model {
    let decls = toy_model_decls("Site", "Site");
    Model::parse(toy_signature_v1(), decls).unwrap()
}

/// A toy example of a ruled-based model (variant 2).
#[cfg(test)]
pub(crate) fn toy_model_v2() -> Model {
    let decls = toy_model_decls("SiteA", "SiteB");
    Model::parse(toy_signature_v2(), decls).unwrap()
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
            PatternTm::tensor([
                PatternTm::res("A", [MorTm::var("r"), MorTm::app("empty", [])]),
                PatternTm::res("B", [MorTm::app("empty", [])]),
            ]),
            PatternTm::let_(
                [ObTm::var("s1"), ObTm::var("s2")],
                MorTm::app("bond", []),
                PatternTm::tensor([
                    PatternTm::res("A", [MorTm::var("r"), MorTm::var("s1")]),
                    PatternTm::res("B", [MorTm::var("s2")]),
                ]),
            ),
        ),
        ModelDecl::rule(
            "phosphorylate",
            [ObTm::var("s")],
            [Ty::sort(site_a)],
            PatternTm::tensor([
                PatternTm::res("A", [MorTm::app("unphos", []), MorTm::var("s")]),
                PatternTm::res("K", []),
            ]),
            PatternTm::tensor([
                PatternTm::res("A", [MorTm::app("phos", []), MorTm::var("s")]),
                PatternTm::res("K", []),
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
            [r] : [Res] ⊢ bondAB [r] : (A [r, empty []], B [empty []]) → let [s1, s2] = bond [] in (A [r, s1], B [s2])
            [s] : [Site] ⊢ phosphorylate [s] : (A [unphos [], s], K []) → (A [phos [], s], K [])
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
            [r] : [Res] ⊢ bondAB [r] : (A [r, empty []], B [empty []]) → let [s1, s2] = bond [] in (A [r, s1], B [s2])
            [s] : [SiteA] ⊢ phosphorylate [s] : (A [unphos [], s], K []) → (A [phos [], s], K [])
        "#]];
        expected.assert_eq(&toy_model_v2().to_string());
    }

    #[test]
    fn species() {
        let model = toy_model_v1();
        let expected = expect![[r#"
            A [unphos [], empty []]
            A [phos [], empty []]
            B [empty []]
            K []
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], A [unphos [], s#2])
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], A [phos [], s#2])
            let (s#1, s#2) = bond [] in (A [phos [], s#1], A [unphos [], s#2])
            let (s#1, s#2) = bond [] in (A [phos [], s#1], A [phos [], s#2])
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], B [s#2])
            let (s#1, s#2) = bond [] in (A [phos [], s#1], B [s#2])
            let (s#1, s#2) = bond [] in (B [s#1], B [s#2])"#]];
        expected.assert_eq(&model.species(2).into_iter().join("\n"));

        let model = toy_model_v2();
        let expected = expect![[r#"
            A [unphos [], emptyA []]
            A [phos [], emptyA []]
            B [emptyB []]
            K []
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], B [s#2])
            let (s#1, s#2) = bond [] in (A [phos [], s#1], B [s#2])"#]];
        expected.assert_eq(&model.species(2).into_iter().join("\n"));
    }
}
