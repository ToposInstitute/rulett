//! Rule-based models.

use itertools::{chain, zip_eq};
use std::{fmt, rc::Rc};
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

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
        if self.rules.contains_key(&name) || self.agents.insert(name, interface).is_some() {
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
        if self.agents.contains_key(&name) || self.rules.insert(name, data).is_some() {
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

    /// Gets the interface of an agent or rule in the model.
    pub fn interface(&self, name: &Name) -> Option<&ObTmJudgment> {
        self.agents
            .get(name)
            .or_else(|| self.rules.get(name).map(|data| &data.interface))
    }

    /// Constructs a model term corresponding to an agent or basic rule.
    fn basic_tm(&self, name: Name, terms: Vec<MorTm>) -> Option<ModelTm> {
        if self.agents.contains_key(&name) {
            Some(ModelTm::Pat(self.agent_tm(name, terms)))
        } else if self.rules.contains_key(&name) {
            Some(ModelTm::Rule(self.rule_tm(name, terms)))
        } else {
            None
        }
    }

    /// Constructs a pattern term corresponding to an agent.
    fn agent_tm(&self, name: Name, terms: Vec<MorTm>) -> PatTm {
        PatTm::Res(name, MorTm::List(terms))
    }

    /// Constructs a rule term corresponding to a basic rule.
    fn rule_tm(&self, name: Name, terms: Vec<MorTm>) -> RuleTm {
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
    pub fn species(&self, max_agents: usize) -> impl Iterator<Item = PatTm> {
        let finder = NetGenerator::new(self);
        (1..=max_agents)
            .flat_map(|n| self.agents.keys().copied().combinations_with_replacement(n))
            .flat_map(move |agents| finder.find(agents))
            .map(|tm| match tm {
                ModelTm::Pat(tm) => tm,
                ModelTm::Rule(_) => unreachable!(),
            })
    }

    /// Derives all transitions generated by the model.
    ///
    /// A *closed rule* is a derived rule with trivial interface. A *transition*
    /// is an indecomposable closed rule, i.e., a closed rule that cannot be
    /// expressed as a nontrivial composite or product of other closed rules.
    pub fn transitions(&self, max_rules: usize) -> impl Iterator<Item = RuleTm> {
        let finder = NetGenerator::new(self);
        (1..=max_rules)
            .flat_map(|n| {
                chain(self.agents.keys(), self.rules.keys())
                    .copied()
                    .combinations_with_replacement(n)
            })
            .filter(|names| names.iter().any(|name| self.rules.contains_key(name)))
            .flat_map(move |names| finder.find(names))
            .map(|tm| match tm {
                ModelTm::Pat(_) => unreachable!(),
                ModelTm::Rule(tm) => tm,
            })
    }
}

/// Wrapper enum for the two kinds of terms in a model.
///
/// This enum exists mainly because the algorithms to generate species or
/// transitions are essentially the same and thus can operate generically over
/// this type instead of being duplicated for each case.
#[derive(Clone, PartialEq, Eq)]
enum ModelTm {
    Pat(PatTm),
    Rule(RuleTm),
}

impl ModelTm {
    fn tensor(tm: ModelTm) -> Self {
        match tm {
            ModelTm::Pat(p) => ModelTm::Pat(PatTm::tensor(p)),
            ModelTm::Rule(r) => ModelTm::Rule(RuleTm::tensor(r)),
        }
    }

    fn list(terms: impl IntoIterator<Item = ModelTm>) -> Self {
        let terms = terms.into_iter().collect_vec();
        if terms.iter().any(|tm| matches!(tm, ModelTm::Rule(_))) {
            let rules = terms
                .into_iter()
                .map(|tm| match tm {
                    ModelTm::Rule(r) => r,
                    ModelTm::Pat(p) => RuleTm { rule: p.clone(), lhs: p.clone(), rhs: p },
                })
                .collect_vec();
            ModelTm::Rule(RuleTm::list(rules))
        } else {
            let patterns = terms.into_iter().map(|tm| match tm {
                ModelTm::Pat(p) => p,
                ModelTm::Rule(_) => unreachable!(),
            });
            ModelTm::Pat(PatTm::list(patterns))
        }
    }

    fn restrict(&self, at: ObTm, along: MorTm) -> Self {
        match self {
            ModelTm::Pat(p) => ModelTm::Pat(p.restrict(at, along)),
            ModelTm::Rule(r) => ModelTm::Rule(r.restrict(at, along)),
        }
    }
}

/// Generates a reaction network from a rule-based model.
struct NetGenerator<'a> {
    /// Model to generate from.
    model: &'a Model,

    /// Index from flattened operation codomains to operation names.
    ///
    /// Note that operations are indexed on *all* unique permutations of their
    /// codomain's sorts.
    cod_index: HashMap<Vec<Name>, Vec<Name>>,
}

/// State maintained by search algorithm for network generation.
#[derive(Clone)]
struct SearchState {
    tm: ModelTm,
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

impl<'a> NetGenerator<'a> {
    fn new(model: &'a Model) -> Self {
        let mut cod_index = HashMap::<_, Vec<_>>::new();
        for (name, _, cod) in model.signature().operations() {
            let sorts = cod.collect_sorts();
            let n = sorts.len();
            for ordering in sorts.into_iter().permutations(n).unique() {
                cod_index.entry(ordering).or_default().push(name);
            }
        }
        Self { model, cod_index }
    }

    fn find(&self, generator_names: Vec<Name>) -> Vec<ModelTm> {
        // Collect all variables, then ensure they are unique.
        let (mut generator_sorts, mut variables) = (Vec::new(), Vec::new());
        for name in &generator_names {
            let interface = self.model.interface(name).unwrap().collect_typed_vars();

            // Degenerate case: if a generator's interface is empty, exit early
            // since any non-trivial product with the generator is decomposable.
            if interface.is_empty() {
                return if generator_names.len() == 1 {
                    vec![self.model.basic_tm(*name, vec![]).unwrap()]
                } else {
                    vec![]
                };
            }

            let (vars, sorts): (Vec<_>, Vec<_>) = interface.into_iter().unzip();
            generator_sorts.push(sorts);
            variables.extend(vars);
        }
        uniquify_names(&mut variables);

        // Build initial term.
        let mut terms = Vec::new();
        let mut interface = Vec::new();
        let mut variables = variables.into_iter();
        for (i, (name, sorts)) in zip_eq(generator_names, generator_sorts).enumerate() {
            let mut vars = Vec::new();
            for sort in sorts {
                let var = variables.next().unwrap();
                interface.push(IntermediateVar { name: var, sort, component: i });
                vars.push(MorTm::var(var));
            }
            terms.push(self.model.basic_tm(name, vars).unwrap());
        }
        let n = terms.len();
        let tm = if n == 1 {
            terms.remove(0)
        } else {
            ModelTm::tensor(ModelTm::list(terms))
        };

        // Initialize the search state, then run the search.
        let uf = Rc::new(QuickUnionUf::new(n));
        let state = SearchState { tm, interface, uf, min_match_idx: 0 };
        let mut results = Vec::new();
        self.recurse(state, &mut results);
        results
    }

    fn recurse(&self, state: SearchState, results: &mut Vec<ModelTm>) {
        let SearchState { interface, tm, uf, min_match_idx } = state;

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
                let (dom, cod) = self.model.signature().interface(op).unwrap();

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
                let state = SearchState {
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

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::expect;

    #[test]
    fn toy_model_v1_generation() {
        let model = toy_model_v1();
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
        expected.assert_eq(&model.to_string());

        let species = expect![[r#"
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
        species.assert_eq(&model.species(2).join("\n"));
    }

    #[test]
    fn toy_model_v2_generation() {
        let model = toy_model_v2();
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
        expected.assert_eq(&model.to_string());

        let species = expect![[r#"
            A [unphos [], emptyA []]
            A [phos [], emptyA []]
            B [emptyB []]
            K []
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], B [s#2])
            let (s#1, s#2) = bond [] in (A [phos [], s#1], B [s#2])"#]];
        species.assert_eq(&model.species(2).join("\n"));

        let rules = expect![[r#"
            bondAB [unphos []] : (A [unphos [], empty []], B [empty []]) → let [s1, s2] = bond [] in (A [unphos [], s1], B [s2])
            bondAB [phos []] : (A [phos [], empty []], B [empty []]) → let [s1, s2] = bond [] in (A [phos [], s1], B [s2])
            phosphorylate [emptyA []] : (A [unphos [], emptyA []], K []) → (A [phos [], emptyA []], K [])
            let (s#1, s#2) = bond [] in (B [s#1], phosphorylate [s#2]) : let (s#1, s#2) = bond [] in (B [s#1], (A [unphos [], s#2], K [])) → let (s#1, s#2) = bond [] in (B [s#1], (A [phos [], s#2], K []))"#]];
        rules.assert_eq(&model.transitions(2).join("\n"));
    }
}
