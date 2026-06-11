//! Network generation from a rule-based model.

use itertools::{chain, zip_eq};
use std::rc::Rc;
use union_find::{QuickUnionUf, UnionBySize, UnionFind};

use super::{model::*, prelude::*, tm::*, ty::*};

/// Wrapper enum for the two kinds of terms in a model.
///
/// This enum exists because the algorithms to generate species or transitions
/// are essentially the same and thus can operate generically over this type
/// instead of being duplicated for each case.
#[derive(Clone, PartialEq, Eq)]
pub enum ModelTm {
    Pat(PatTm),
    Rule(RuleTm),
}

impl ModelTm {
    /// Deconstructor for [`Pat`](Self::Pat) variant.
    fn take_pattern(self) -> Option<PatTm> {
        match self {
            Self::Pat(tm) => Some(tm),
            Self::Rule(_) => None,
        }
    }

    /// Deconstructor for the [`Rule`](Self::Rule) variant.
    fn take_rule(self) -> Option<RuleTm> {
        match self {
            Self::Rule(tm) => Some(tm),
            Self::Pat(_) => None,
        }
    }

    /// Constructs a model term corresponding to an agent or basic rule.
    fn basic(model: &Model, name: Name, terms: Vec<MorTm>) -> Option<Self> {
        if model.has_agent(&name) {
            Some(ModelTm::Pat(model.agent_tm(name, terms)))
        } else if model.has_rule(&name) {
            Some(ModelTm::Rule(model.rule_tm(name, terms)))
        } else {
            None
        }
    }

    /// Constructs an application of the tensor product to a model term.
    fn tensor(tm: ModelTm) -> Self {
        match tm {
            ModelTm::Pat(p) => ModelTm::Pat(PatTm::tensor(p)),
            ModelTm::Rule(r) => ModelTm::Rule(RuleTm::tensor(r)),
        }
    }

    /// Constructs a list of model terms.
    fn list(terms: impl IntoIterator<Item = ModelTm>) -> Self {
        let terms = terms.into_iter().collect_vec();
        if terms.iter().any(|tm| matches!(tm, ModelTm::Rule(_))) {
            // If any term is a rule, promote them all to rules.
            let rules = terms
                .into_iter()
                .map(|tm| match tm {
                    ModelTm::Rule(r) => r,
                    ModelTm::Pat(p) => RuleTm { rule: p.clone(), lhs: p.clone(), rhs: p },
                })
                .collect_vec();
            ModelTm::Rule(RuleTm::list(rules))
        } else {
            // Otherwise, we have a list of patterns.
            let patterns = terms.into_iter().map(|tm| tm.take_pattern().unwrap());
            ModelTm::Pat(PatTm::list(patterns))
        }
    }

    /// Restricts the model term at free variables along a morphism term.
    fn restrict(&self, at: ObTm, along: MorTm) -> Self {
        match self {
            ModelTm::Pat(p) => ModelTm::Pat(p.restrict(at, along)),
            ModelTm::Rule(r) => ModelTm::Rule(r.restrict(at, along)),
        }
    }
}

/// Generates a reaction network from a rule-based model.
pub struct NetGenerator<'a> {
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
    /// Current term.
    tm: ModelTm,

    /// Current list of free variables.
    interface: Vec<IntermediateVar>,

    /// Generator of new free variables.
    name_gen: NameGenerator,

    /// Current partition of (indexes of) original list of free variables.
    ///
    /// Wrap the union find in `Rc` since we don't have to mutate it at each
    /// branch point, only when restricting along an operation of co-arity >= 2.
    uf: Rc<QuickUnionUf<UnionBySize>>,

    /// Set of free variables that have been seen but not substituted.
    seen: im::HashSet<Name>,
}

#[derive(Clone, Copy, Debug)]
struct IntermediateVar {
    name: Name,
    sort: Name,
    component: usize,
}

impl<'a> NetGenerator<'a> {
    /// Constructs a network generator for the given model.
    pub fn new(model: &'a Model) -> Self {
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

    /// Generates species from the model up to a specified size.
    ///
    /// A *closed pattern* is a pattern with trivial interface. A *species* is
    /// an indecomposable closed pattern, i.e., a closed pattern that cannot be
    /// expressed as a non-trivial product of other closed patterns.
    pub fn species(&self, max_agents: usize) -> impl Iterator<Item = PatTm> {
        (1..=max_agents)
            .flat_map(|n| self.model.agent_names().combinations_with_replacement(n))
            .flat_map(|agents| self.find(agents))
            .map(|tm| tm.take_pattern().unwrap_or_else(|| unreachable!()))
    }

    /// Generates transitions from the model up to a specified size.
    ///
    /// A *closed rule* is a derived rule with trivial interface. A *transition*
    /// is an indecomposable closed rule, i.e., a closed rule that cannot be
    /// expressed as a nontrivial composite or product of other closed rules.
    pub fn transitions(&self, max_rules: usize) -> impl Iterator<Item = RuleTm> {
        (1..=max_rules)
            .flat_map(|n| {
                chain(self.model.agent_names(), self.model.rule_names())
                    .combinations_with_replacement(n)
            })
            .filter(|names| names.iter().any(|name| self.model.has_rule(name)))
            .flat_map(|names| self.find(names))
            .map(|tm| tm.take_rule().unwrap_or_else(|| unreachable!()))
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
                    vec![ModelTm::basic(self.model, *name, vec![]).unwrap()]
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
            terms.push(ModelTm::basic(self.model, name, vars).unwrap());
        }
        let n = terms.len();
        let tm = if n == 1 {
            terms.remove(0)
        } else {
            ModelTm::tensor(ModelTm::list(terms))
        };

        // Initialize the search state, then run the search.
        let state = SearchState {
            tm,
            interface,
            name_gen: Default::default(),
            uf: Rc::new(QuickUnionUf::new(n)),
            seen: Default::default(),
        };
        let mut results = Vec::new();
        self.recurse(state, &mut results);
        results
    }

    fn recurse(&self, state: SearchState, results: &mut Vec<ModelTm>) {
        let SearchState { interface, tm, name_gen, uf, seen } = state;

        // Success condition: found a closed term.
        if interface.is_empty() {
            results.push(tm);
            return;
        }

        for idxs in (0..interface.len()).powerset() {
            // To avoid duplicate species, skip any matches that do not include
            // at least one free variable that has not already been seen.
            //
            // As a degenerate case, do not restrict along co-nullary operations
            // as that causes infinite blow-up. Such operations, which include
            // [scalars](https://ncatlab.org/nlab/show/monoidal+category#scalars),
            // also seem pointless, but perhaps they're good for something?
            if idxs.iter().all(|&i| seen.contains(&interface[i].name)) {
                continue;
            }

            // Get co-applicable operations, bailing early if there are none.
            let sorts = idxs.iter().map(|&i| interface[i].sort).collect_vec();
            let Some(operations) = self.cod_index.get(&sorts).filter(|ops| !ops.is_empty()) else {
                continue;
            };

            // Mark as seen all free variables up to the last matched variable
            // (unless they're already eliminated by the match).
            let mut seen = seen.clone();
            let last_idx = idxs.iter().max().unwrap();
            for (i, var) in interface.iter().enumerate().take(last_idx + 1) {
                if !idxs.contains(&i) {
                    seen.insert(var.name);
                }
            }

            // Union components involved in restricting along these indices.
            let mut uf = uf.clone();
            let mut has_merged = false;
            let mut components = idxs.iter().map(|&i| interface[i].component);
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

                let mut name_gen = name_gen.clone();
                let mut interface_added = dom
                    .collect_sorts()
                    .into_iter()
                    .map(|sort| {
                        let name = name_gen.gensym(&sort.to_lowercase());
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

                let vars = idxs.iter().map(|&i| ObTm::var(interface[i].name));
                let restrict_at = if matches!(cod, Ty::Sort(_)) {
                    vars.exactly_one().unwrap()
                } else {
                    ObTm::tensor(ObTm::list(vars))
                };

                let args = MorTm::list(interface_added.iter().map(|var| MorTm::var(var.name)));
                let restrict_along = MorTm::app(*op, args);

                let mut interface = interface_kept.clone();
                interface.append(&mut interface_added);
                let state = SearchState {
                    tm: tm.restrict(restrict_at, restrict_along),
                    interface,
                    name_gen,
                    uf: uf.clone(),
                    seen: seen.clone(),
                };
                self.recurse(state, results)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{super::model, super::theory::*, *};
    use expect_test::expect;

    #[test]
    fn toy_model_v1() {
        let model = model::toy_model_v1();
        let generator = NetGenerator::new(&model);

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
        species.assert_eq(&generator.species(2).join("\n"));

        let transitions = expect![[r#"
            bondAB [unphos []]
              : (A [unphos [], empty []], B [empty []])
              → let [s1, s2] = bond [] in (A [unphos [], s1], B [s2])
            bondAB [phos []]
              : (A [phos [], empty []], B [empty []])
              → let [s1, s2] = bond [] in (A [phos [], s1], B [s2])
            phosphorylate [empty []]
              : (A [unphos [], empty []], K [])
              → (A [phos [], empty []], K [])
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], phosphorylate [s#2])
              : let (s#1, s#2) = bond [] in (A [unphos [], s#1], (A [unphos [], s#2], K []))
              → let (s#1, s#2) = bond [] in (A [unphos [], s#1], (A [phos [], s#2], K []))
            let (s#1, s#2) = bond [] in (A [phos [], s#1], phosphorylate [s#2])
              : let (s#1, s#2) = bond [] in (A [phos [], s#1], (A [unphos [], s#2], K []))
              → let (s#1, s#2) = bond [] in (A [phos [], s#1], (A [phos [], s#2], K []))
            let (s#1, s#2) = bond [] in (B [s#1], phosphorylate [s#2])
              : let (s#1, s#2) = bond [] in (B [s#1], (A [unphos [], s#2], K []))
              → let (s#1, s#2) = bond [] in (B [s#1], (A [phos [], s#2], K []))
            let (s#1, s#2) = bond [] in (phosphorylate [s#1], phosphorylate [s#2])
              : let (s#1, s#2) = bond [] in
                ((A [unphos [], s#1], K []), (A [unphos [], s#2], K []))
              → let (s#1, s#2) = bond [] in
                ((A [phos [], s#1], K []), (A [phos [], s#2], K []))"#]];
        transitions.assert_eq(&generator.transitions(2).join("\n"));
    }

    #[test]
    fn toy_model_v2() {
        let model = model::toy_model_v2();
        let generator = NetGenerator::new(&model);

        let species = expect![[r#"
            A [unphos [], emptyA []]
            A [phos [], emptyA []]
            B [emptyB []]
            K []
            let (s#1, s#2) = bond [] in (A [unphos [], s#1], B [s#2])
            let (s#1, s#2) = bond [] in (A [phos [], s#1], B [s#2])"#]];
        species.assert_eq(&generator.species(2).join("\n"));

        let transitions = expect![[r#"
            bondAB [unphos []]
              : (A [unphos [], empty []], B [empty []])
              → let [s1, s2] = bond [] in (A [unphos [], s1], B [s2])
            bondAB [phos []]
              : (A [phos [], empty []], B [empty []])
              → let [s1, s2] = bond [] in (A [phos [], s1], B [s2])
            phosphorylate [emptyA []]
              : (A [unphos [], emptyA []], K [])
              → (A [phos [], emptyA []], K [])
            let (s#1, s#2) = bond [] in (B [s#1], phosphorylate [s#2])
              : let (s#1, s#2) = bond [] in (B [s#1], (A [unphos [], s#2], K []))
              → let (s#1, s#2) = bond [] in (B [s#1], (A [phos [], s#2], K []))"#]];
        transitions.assert_eq(&generator.transitions(2).join("\n"));
    }

    #[test]
    fn expose_site() {
        type SigDecl = SignatureDecl;
        let sig = Signature::parse([
            SigDecl::sort("SiteA"),
            SigDecl::sort("SiteB"),
            SigDecl::sort("MaybeSiteB"),
            SigDecl::operation("noSite", [], Ty::sort("MaybeSiteB")),
            SigDecl::operation("hasSite", [Ty::sort("SiteB")], Ty::sort("MaybeSiteB")),
            SigDecl::operation("bond", [], Ty::tensor([Ty::sort("SiteA"), Ty::sort("SiteB")])),
        ])
        .unwrap();
        let decls = [
            ModelDecl::agent("A", [ObTm::var("s")], [Ty::sort("SiteA")]),
            ModelDecl::agent("B", [ObTm::var("s")], [Ty::sort("MaybeSiteB")]),
        ];
        let model = Model::parse(sig, decls).unwrap();

        let species = expect![[r#"
            B [noSite []]
            let (s#1, ##siteb#1) = bond [] in (A [s#1], B [hasSite [##siteb#1]])"#]];
        species.assert_eq(&NetGenerator::new(&model).species(2).join("\n"));
    }

    fn toy_model_single_agent() {
        let model = model::toy_model_single_agent();
        let generator = NetGenerator::new(&model);

        let species = expect![[r#"
            M [iota_A [ground_A []]]
            M [iota_B [ground_B []]]
            M [iota_K [ground_K []]]
            M [iota_SiteA [emptyA []]]
            M [iota_SiteB [emptyB []]]
            M [iota_Res [phos []]]
            M [iota_Res [unphos []]]
            let (##sitea#27, ##siteb#32) = bond [] in
              (M [iota_SiteA [##sitea#27]], M [iota_SiteB [##siteb#32]])
            let (##siteb#34, ##sitea#38) = bond [] in
              (M [iota_SiteB [##siteb#34]], M [iota_SiteA [##sitea#38]])"#]];
        species.assert_eq(&generator.species(2).join("\n"));

        let transitions = expect![[r#"
            bondAB [phos []]
              : (
                (M iota_A, M iota_SiteB empty [], M iota_Res r []),
                (M iota_B, M iota_SiteA empty [])
              )
              → let [s1, s2] = bond [] in
                (
                  (M iota_A, M iota_SiteB s2 [], M iota_Res r []),
                  (M iota_B, M iota_SiteA s1 [])
                )
            bondAB [unphos []]
              : (
                (M iota_A, M iota_SiteB empty [], M iota_Res r []),
                (M iota_B, M iota_SiteA empty [])
              )
              → let [s1, s2] = bond [] in
                (
                  (M iota_A, M iota_SiteB s2 [], M iota_Res r []),
                  (M iota_B, M iota_SiteA s1 [])
                )
            phosphorylate [emptyB []]
              : ((M iota_A, M iota_SiteB s [], M iota_Res unphos []), (M iota_K))
              → ((M iota_A, M iota_SiteB s [], M iota_Res phos []), (M iota_K))
            let (s, ##sitea#63) = bond [] in
                (M [iota_SiteA [##sitea#63]], phosphorylate [s])
              : let (s, ##sitea#63) = bond [] in
                (
                  M [iota_SiteA [##sitea#63]],
                  ((M iota_A, M iota_SiteB s [], M iota_Res unphos []), (M iota_K))
                )
              → let (s, ##sitea#63) = bond [] in
                (
                  M [iota_SiteA [##sitea#63]],
                  ((M iota_A, M iota_SiteB s [], M iota_Res phos []), (M iota_K))
                )"#]];
        transitions.assert_eq(&generator.transitions(2).join("\n"));
    }

    #[test]
    fn toy_model_emergent_agent() {
        let model = model::toy_model_emergent_agent();
        let generator = NetGenerator::new(&model);

        let species = expect![[r#"
            A [e_B [], e_C []]
            B [e_A [], e_C []]
            C [e_AB [], e_AB []]
            let (ab, ba) = bond_AB [] in (A [ab, e_C []], B [ba, e_C []])
            let (ac, ca) = bond_C [] in (A [e_B [], ac], C [ca, e_AB []])
            let (ac, cb) = bond_C [] in (A [e_B [], ac], C [e_AB [], cb])
            let (bc, ca) = bond_C [] in (B [e_A [], bc], C [ca, e_AB []])
            let (bc, cb) = bond_C [] in (B [e_A [], bc], C [e_AB [], cb])
            let (ac#2, cb) = bond_C [] in
              let (ac#1, ca) = bond_C [] in (A [e_B [], ac#1], A [e_B [], ac#2], C [ca, cb])
            let (ac#2, ca) = bond_C [] in
              let (ac#1, cb) = bond_C [] in (A [e_B [], ac#1], A [e_B [], ac#2], C [ca, cb])
            let (bc, cb) = bond_C [] in
              let (ac, ca) = bond_C [] in (A [e_B [], ac], B [e_A [], bc], C [ca, cb])
            let (bc, ca) = bond_C [] in
              let (ac, cb) = bond_C [] in (A [e_B [], ac], B [e_A [], bc], C [ca, cb])
            let (bc, ca) = bond_C [] in
              let (ab, ba) = bond_AB [] in (A [ab, e_C []], B [ba, bc], C [ca, e_AB []])
            let (bc, cb) = bond_C [] in
              let (ab, ba) = bond_AB [] in (A [ab, e_C []], B [ba, bc], C [e_AB [], cb])
            let (ac, ca) = bond_C [] in
              let (ab, ba) = bond_AB [] in (A [ab, ac], B [ba, e_C []], C [ca, e_AB []])
            let (bc, cb) = bond_C [] in
              let (ac, ca) = bond_C [] in
                let (ab, ba) = bond_AB [] in (A [ab, ac], B [ba, bc], C [ca, cb])
            let (ac, cb) = bond_C [] in
              let (ab, ba) = bond_AB [] in (A [ab, ac], B [ba, e_C []], C [e_AB [], cb])
            let (bc, ca) = bond_C [] in
              let (ac, cb) = bond_C [] in
                let (ab, ba) = bond_AB [] in (A [ab, ac], B [ba, bc], C [ca, cb])
            let (bc#2, cb) = bond_C [] in
              let (bc#1, ca) = bond_C [] in (B [e_A [], bc#1], B [e_A [], bc#2], C [ca, cb])
            let (bc#2, ca) = bond_C [] in
              let (bc#1, cb) = bond_C [] in (B [e_A [], bc#1], B [e_A [], bc#2], C [ca, cb])"#]];
        species.assert_eq(&generator.species(3).join("\n"));

        let transitions = expect![[r#"
            R_dimerization [e_C [], e_C []]
              : (A [e_B], B [e_A])
              → let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C])
            R_trimerization []
              : (let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]), C [e_AB, e_AB])
              → let [bc, cb] = bond [] in
                let [ac, ca] = bond [] in
                  (let [ab, ba] = bond [] in (A [ab, c], B [ba, c]), C [ca, cb])
            let (cb#1, ca#2) = bond_C [] in
                (C [e_AB [], cb#1], R_dimerization [ca#2, e_C []])
              : let (cb#1, ca#2) = bond_C [] in (C [e_AB [], cb#1], (A [e_B], B [e_A]))
              → let (cb#1, ca#2) = bond_C [] in
                (C [e_AB [], cb#1], let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]))
            let (cb#1, cb#2) = bond_C [] in
                (C [e_AB [], cb#1], R_dimerization [e_C [], cb#2])
              : let (cb#1, cb#2) = bond_C [] in (C [e_AB [], cb#1], (A [e_B], B [e_A]))
              → let (cb#1, cb#2) = bond_C [] in
                (C [e_AB [], cb#1], let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]))
            let (ca#1, ca#2) = bond_C [] in
                (C [ca#1, e_AB []], R_dimerization [ca#2, e_C []])
              : let (ca#1, ca#2) = bond_C [] in (C [ca#1, e_AB []], (A [e_B], B [e_A]))
              → let (ca#1, ca#2) = bond_C [] in
                (C [ca#1, e_AB []], let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]))
            let (cb#1, cb#2) = bond_C [] in
                let (ca#1, ca#2) = bond_C [] in
                  (C [ca#1, cb#1], R_dimerization [ca#2, cb#2])
              : let (cb#1, cb#2) = bond_C [] in
                let (ca#1, ca#2) = bond_C [] in (C [ca#1, cb#1], (A [e_B], B [e_A]))
              → let (cb#1, cb#2) = bond_C [] in
                let (ca#1, ca#2) = bond_C [] in
                  (C [ca#1, cb#1], let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]))
            let (ca#1, cb#2) = bond_C [] in
                (C [ca#1, e_AB []], R_dimerization [e_C [], cb#2])
              : let (ca#1, cb#2) = bond_C [] in (C [ca#1, e_AB []], (A [e_B], B [e_A]))
              → let (ca#1, cb#2) = bond_C [] in
                (C [ca#1, e_AB []], let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]))
            let (cb#1, ca#2) = bond_C [] in
                let (ca#1, cb#2) = bond_C [] in
                  (C [ca#1, cb#1], R_dimerization [ca#2, cb#2])
              : let (cb#1, ca#2) = bond_C [] in
                let (ca#1, cb#2) = bond_C [] in (C [ca#1, cb#1], (A [e_B], B [e_A]))
              → let (cb#1, ca#2) = bond_C [] in
                let (ca#1, cb#2) = bond_C [] in
                  (C [ca#1, cb#1], let [s1, s2] = bond [] in (A [s1, e_C], B [s2, e_C]))"#]];
        transitions.assert_eq(&generator.transitions(2).join("\n")); // TODO: consider stronger typing to avoid explosion of transitions
    }

    #[test]
    fn toy_model_directionality() {
        let model = model::toy_model_directionality();
        let generator = NetGenerator::new(&model);

        let species = expect![[r#"
            A [e_h [], e_C [], e_t []]
            let (ah, at) = bond_AB [] in A [ah, e_C [], at]
            B [e_h [], e_C [], e_t []]
            let (bh, bt) = bond_AB [] in B [bh, e_C [], bt]
            C [e_ABh [], e_ABt []]
            let (at#1, ah#2) = bond_AB [] in
              (A [e_h [], e_C [], at#1], A [ah#2, e_C [], e_t []])
            let (ah#1, at#2) = bond_AB [] in
              (A [ah#1, e_C [], e_t []], A [e_h [], e_C [], at#2])
            let (at#1, ah#2) = bond_AB [] in
              let (ah#1, at#2) = bond_AB [] in
                (A [ah#1, e_C [], at#1], A [ah#2, e_C [], at#2])
            let (at, bh) = bond_AB [] in (A [e_h [], e_C [], at], B [bh, e_C [], e_t []])
            let (ah, bt) = bond_AB [] in (A [ah, e_C [], e_t []], B [e_h [], e_C [], bt])
            let (at, bh) = bond_AB [] in
              let (ah, bt) = bond_AB [] in (A [ah, e_C [], at], B [bh, e_C [], bt])
            let (ac, ch) = bond_Ch [] in (A [e_h [], ac, e_t []], C [ch, e_ABt []])
            let (ac, ct) = bond_Ct [] in (A [e_h [], ac, e_t []], C [e_ABh [], ct])
            let (ac, ch) = bond_Ch [] in
              let (ah, at) = bond_AB [] in (A [ah, ac, at], C [ch, e_ABt []])
            let (ac, ct) = bond_Ct [] in
              let (ah, at) = bond_AB [] in (A [ah, ac, at], C [e_ABh [], ct])
            let (bt#1, bh#2) = bond_AB [] in
              (B [e_h [], e_C [], bt#1], B [bh#2, e_C [], e_t []])
            let (bh#1, bt#2) = bond_AB [] in
              (B [bh#1, e_C [], e_t []], B [e_h [], e_C [], bt#2])
            let (bt#1, bh#2) = bond_AB [] in
              let (bh#1, bt#2) = bond_AB [] in
                (B [bh#1, e_C [], bt#1], B [bh#2, e_C [], bt#2])
            let (bc, ch) = bond_Ch [] in (B [e_h [], bc, e_t []], C [ch, e_ABt []])
            let (bc, ct) = bond_Ct [] in (B [e_h [], bc, e_t []], C [e_ABh [], ct])
            let (bc, ch) = bond_Ch [] in
              let (bh, bt) = bond_AB [] in (B [bh, bc, bt], C [ch, e_ABt []])
            let (bc, ct) = bond_Ct [] in
              let (bh, bt) = bond_AB [] in (B [bh, bc, bt], C [e_ABh [], ct])"#]];
        species.assert_eq(&generator.species(2).join("\n")); // Note that this model allows infinite polymerization

        let transitions = expect![[r#"
            R_dimerization [e_h [], e_C [], e_C [], e_t []]
              : (A [e_h [], e_C [], e_t], B [e_h, e_C [], e_t []])
              → let [s1, s2] = bond_AB [] in
                (A [e_h [], e_C [], s1], B [s2, e_C [], e_t []])
            let (id_head, id_tail) = bond_AB [] in
                R_dimerization [id_head, e_C [], e_C [], id_tail]
              : let (id_head, id_tail) = bond_AB [] in
                (A [id_head, e_C [], e_t], B [e_h, e_C [], id_tail])
              → let (id_head, id_tail) = bond_AB [] in
                let [s1, s2] = bond_AB [] in
                  (A [id_head, e_C [], s1], B [s2, e_C [], id_tail])
            R_trimerization [e_h [], e_t []]
              : (
                let [s1, s2] = bond [] in (A [e_h [], e_C, s1], B [s2, e_C, e_t []]),
                C [e_Ch, e_Ct]
              )
              → let [ac, ca] = bond_Ch [] in
                let [bc, cb] = bond_Ct [] in
                  let [ab, ba] = bond_AB [] in
                    (A [e_h [], ac, ab], B [ba, bc, e_t []], C [ca, cb])
            let (id_head, id_tail) = bond_AB [] in R_trimerization [id_head, id_tail]
              : let (id_head, id_tail) = bond_AB [] in
                (
                  let [s1, s2] = bond [] in (A [id_head, e_C, s1], B [s2, e_C, id_tail]),
                  C [e_Ch, e_Ct]
                )
              → let (id_head, id_tail) = bond_AB [] in
                let [ac, ca] = bond_Ch [] in
                  let [bc, cb] = bond_Ct [] in
                    let [ab, ba] = bond_AB [] in
                      (A [id_head, ac, ab], B [ba, bc, id_tail], C [ca, cb])
            let (at, id_head) = bond_AB [] in
                (A [e_h [], e_C [], at], R_dimerization [id_head, e_C [], e_C [], e_t []])
              : let (at, id_head) = bond_AB [] in
                (
                  A [e_h [], e_C [], at],
                  (A [id_head, e_C [], e_t], B [e_h, e_C [], e_t []])
                )
              → let (at, id_head) = bond_AB [] in
                (
                  A [e_h [], e_C [], at],
                  let [s1, s2] = bond_AB [] in
                    (A [id_head, e_C [], s1], B [s2, e_C [], e_t []])
                )
            let (ah, id_tail) = bond_AB [] in
                (A [ah, e_C [], e_t []], R_dimerization [e_h [], e_C [], e_C [], id_tail])
              : let (ah, id_tail) = bond_AB [] in
                (
                  A [ah, e_C [], e_t []],
                  (A [e_h [], e_C [], e_t], B [e_h, e_C [], id_tail])
                )
              → let (ah, id_tail) = bond_AB [] in
                (
                  A [ah, e_C [], e_t []],
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, e_C [], id_tail])
                )
            let (at, id_head) = bond_AB [] in
                let (ah, id_tail) = bond_AB [] in
                  (A [ah, e_C [], at], R_dimerization [id_head, e_C [], e_C [], id_tail])
              : let (at, id_head) = bond_AB [] in
                let (ah, id_tail) = bond_AB [] in
                  (A [ah, e_C [], at], (A [id_head, e_C [], e_t], B [e_h, e_C [], id_tail]))
              → let (at, id_head) = bond_AB [] in
                let (ah, id_tail) = bond_AB [] in
                  (
                    A [ah, e_C [], at],
                    let [s1, s2] = bond_AB [] in
                      (A [id_head, e_C [], s1], B [s2, e_C [], id_tail])
                  )
            let (at, id_head) = bond_AB [] in
                (A [e_h [], e_C [], at], R_trimerization [id_head, e_t []])
              : let (at, id_head) = bond_AB [] in
                (
                  A [e_h [], e_C [], at],
                  (
                    let [s1, s2] = bond [] in (A [id_head, e_C, s1], B [s2, e_C, e_t []]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (at, id_head) = bond_AB [] in
                (
                  A [e_h [], e_C [], at],
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [id_head, ac, ab], B [ba, bc, e_t []], C [ca, cb])
                )
            let (ah, id_tail) = bond_AB [] in
                (A [ah, e_C [], e_t []], R_trimerization [e_h [], id_tail])
              : let (ah, id_tail) = bond_AB [] in
                (
                  A [ah, e_C [], e_t []],
                  (
                    let [s1, s2] = bond [] in (A [e_h [], e_C, s1], B [s2, e_C, id_tail]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (ah, id_tail) = bond_AB [] in
                (
                  A [ah, e_C [], e_t []],
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [e_h [], ac, ab], B [ba, bc, id_tail], C [ca, cb])
                )
            let (at, id_head) = bond_AB [] in
                let (ah, id_tail) = bond_AB [] in
                  (A [ah, e_C [], at], R_trimerization [id_head, id_tail])
              : let (at, id_head) = bond_AB [] in
                let (ah, id_tail) = bond_AB [] in
                  (
                    A [ah, e_C [], at],
                    (
                      let [s1, s2] = bond [] in
                        (A [id_head, e_C, s1], B [s2, e_C, id_tail]),
                      C [e_Ch, e_Ct]
                    )
                  )
              → let (at, id_head) = bond_AB [] in
                let (ah, id_tail) = bond_AB [] in
                  (
                    A [ah, e_C [], at],
                    let [ac, ca] = bond_Ch [] in
                      let [bc, cb] = bond_Ct [] in
                        let [ab, ba] = bond_AB [] in
                          (A [id_head, ac, ab], B [ba, bc, id_tail], C [ca, cb])
                  )
            let (bt, id_head) = bond_AB [] in
                (B [e_h [], e_C [], bt], R_dimerization [id_head, e_C [], e_C [], e_t []])
              : let (bt, id_head) = bond_AB [] in
                (
                  B [e_h [], e_C [], bt],
                  (A [id_head, e_C [], e_t], B [e_h, e_C [], e_t []])
                )
              → let (bt, id_head) = bond_AB [] in
                (
                  B [e_h [], e_C [], bt],
                  let [s1, s2] = bond_AB [] in
                    (A [id_head, e_C [], s1], B [s2, e_C [], e_t []])
                )
            let (bh, id_tail) = bond_AB [] in
                (B [bh, e_C [], e_t []], R_dimerization [e_h [], e_C [], e_C [], id_tail])
              : let (bh, id_tail) = bond_AB [] in
                (
                  B [bh, e_C [], e_t []],
                  (A [e_h [], e_C [], e_t], B [e_h, e_C [], id_tail])
                )
              → let (bh, id_tail) = bond_AB [] in
                (
                  B [bh, e_C [], e_t []],
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, e_C [], id_tail])
                )
            let (bt, id_head) = bond_AB [] in
                let (bh, id_tail) = bond_AB [] in
                  (B [bh, e_C [], bt], R_dimerization [id_head, e_C [], e_C [], id_tail])
              : let (bt, id_head) = bond_AB [] in
                let (bh, id_tail) = bond_AB [] in
                  (B [bh, e_C [], bt], (A [id_head, e_C [], e_t], B [e_h, e_C [], id_tail]))
              → let (bt, id_head) = bond_AB [] in
                let (bh, id_tail) = bond_AB [] in
                  (
                    B [bh, e_C [], bt],
                    let [s1, s2] = bond_AB [] in
                      (A [id_head, e_C [], s1], B [s2, e_C [], id_tail])
                  )
            let (bt, id_head) = bond_AB [] in
                (B [e_h [], e_C [], bt], R_trimerization [id_head, e_t []])
              : let (bt, id_head) = bond_AB [] in
                (
                  B [e_h [], e_C [], bt],
                  (
                    let [s1, s2] = bond [] in (A [id_head, e_C, s1], B [s2, e_C, e_t []]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (bt, id_head) = bond_AB [] in
                (
                  B [e_h [], e_C [], bt],
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [id_head, ac, ab], B [ba, bc, e_t []], C [ca, cb])
                )
            let (bh, id_tail) = bond_AB [] in
                (B [bh, e_C [], e_t []], R_trimerization [e_h [], id_tail])
              : let (bh, id_tail) = bond_AB [] in
                (
                  B [bh, e_C [], e_t []],
                  (
                    let [s1, s2] = bond [] in (A [e_h [], e_C, s1], B [s2, e_C, id_tail]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (bh, id_tail) = bond_AB [] in
                (
                  B [bh, e_C [], e_t []],
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [e_h [], ac, ab], B [ba, bc, id_tail], C [ca, cb])
                )
            let (bt, id_head) = bond_AB [] in
                let (bh, id_tail) = bond_AB [] in
                  (B [bh, e_C [], bt], R_trimerization [id_head, id_tail])
              : let (bt, id_head) = bond_AB [] in
                let (bh, id_tail) = bond_AB [] in
                  (
                    B [bh, e_C [], bt],
                    (
                      let [s1, s2] = bond [] in
                        (A [id_head, e_C, s1], B [s2, e_C, id_tail]),
                      C [e_Ch, e_Ct]
                    )
                  )
              → let (bt, id_head) = bond_AB [] in
                let (bh, id_tail) = bond_AB [] in
                  (
                    B [bh, e_C [], bt],
                    let [ac, ca] = bond_Ch [] in
                      let [bc, cb] = bond_Ct [] in
                        let [ab, ba] = bond_AB [] in
                          (A [id_head, ac, ab], B [ba, bc, id_tail], C [ca, cb])
                  )
            let (ct, id_Site_C1) = bond_Ct [] in
                (C [e_ABh [], ct], R_dimerization [e_h [], id_Site_C1, e_C [], e_t []])
              : let (ct, id_Site_C1) = bond_Ct [] in
                (C [e_ABh [], ct], (A [e_h [], id_Site_C1, e_t], B [e_h, e_C [], e_t []]))
              → let (ct, id_Site_C1) = bond_Ct [] in
                (
                  C [e_ABh [], ct],
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], id_Site_C1, s1], B [s2, e_C [], e_t []])
                )
            let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C1) = bond_Ct [] in
                  (C [e_ABh [], ct], R_dimerization [id_head, id_Site_C1, e_C [], id_tail])
              : let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C1) = bond_Ct [] in
                  (
                    C [e_ABh [], ct],
                    (A [id_head, id_Site_C1, e_t], B [e_h, e_C [], id_tail])
                  )
              → let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C1) = bond_Ct [] in
                  (
                    C [e_ABh [], ct],
                    let [s1, s2] = bond_AB [] in
                      (A [id_head, id_Site_C1, s1], B [s2, e_C [], id_tail])
                  )
            let (ct, id_Site_C2) = bond_Ct [] in
                (C [e_ABh [], ct], R_dimerization [e_h [], e_C [], id_Site_C2, e_t []])
              : let (ct, id_Site_C2) = bond_Ct [] in
                (C [e_ABh [], ct], (A [e_h [], e_C [], e_t], B [e_h, id_Site_C2, e_t []]))
              → let (ct, id_Site_C2) = bond_Ct [] in
                (
                  C [e_ABh [], ct],
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, id_Site_C2, e_t []])
                )
            let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C2) = bond_Ct [] in
                  (C [e_ABh [], ct], R_dimerization [id_head, e_C [], id_Site_C2, id_tail])
              : let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C2) = bond_Ct [] in
                  (
                    C [e_ABh [], ct],
                    (A [id_head, e_C [], e_t], B [e_h, id_Site_C2, id_tail])
                  )
              → let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C2) = bond_Ct [] in
                  (
                    C [e_ABh [], ct],
                    let [s1, s2] = bond_AB [] in
                      (A [id_head, e_C [], s1], B [s2, id_Site_C2, id_tail])
                  )
            let (ch, id_Site_C1) = bond_Ch [] in
                (C [ch, e_ABt []], R_dimerization [e_h [], id_Site_C1, e_C [], e_t []])
              : let (ch, id_Site_C1) = bond_Ch [] in
                (C [ch, e_ABt []], (A [e_h [], id_Site_C1, e_t], B [e_h, e_C [], e_t []]))
              → let (ch, id_Site_C1) = bond_Ch [] in
                (
                  C [ch, e_ABt []],
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], id_Site_C1, s1], B [s2, e_C [], e_t []])
                )
            let (id_head, id_tail) = bond_AB [] in
                let (ch, id_Site_C1) = bond_Ch [] in
                  (C [ch, e_ABt []], R_dimerization [id_head, id_Site_C1, e_C [], id_tail])
              : let (id_head, id_tail) = bond_AB [] in
                let (ch, id_Site_C1) = bond_Ch [] in
                  (
                    C [ch, e_ABt []],
                    (A [id_head, id_Site_C1, e_t], B [e_h, e_C [], id_tail])
                  )
              → let (id_head, id_tail) = bond_AB [] in
                let (ch, id_Site_C1) = bond_Ch [] in
                  (
                    C [ch, e_ABt []],
                    let [s1, s2] = bond_AB [] in
                      (A [id_head, id_Site_C1, s1], B [s2, e_C [], id_tail])
                  )
            let (ct, id_Site_C2) = bond_Ct [] in
                let (ch, id_Site_C1) = bond_Ch [] in
                  (C [ch, ct], R_dimerization [e_h [], id_Site_C1, id_Site_C2, e_t []])
              : let (ct, id_Site_C2) = bond_Ct [] in
                let (ch, id_Site_C1) = bond_Ch [] in
                  (C [ch, ct], (A [e_h [], id_Site_C1, e_t], B [e_h, id_Site_C2, e_t []]))
              → let (ct, id_Site_C2) = bond_Ct [] in
                let (ch, id_Site_C1) = bond_Ch [] in
                  (
                    C [ch, ct],
                    let [s1, s2] = bond_AB [] in
                      (A [e_h [], id_Site_C1, s1], B [s2, id_Site_C2, e_t []])
                  )
            let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C2) = bond_Ct [] in
                  let (ch, id_Site_C1) = bond_Ch [] in
                    (C [ch, ct], R_dimerization [id_head, id_Site_C1, id_Site_C2, id_tail])
              : let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C2) = bond_Ct [] in
                  let (ch, id_Site_C1) = bond_Ch [] in
                    (
                      C [ch, ct],
                      (A [id_head, id_Site_C1, e_t], B [e_h, id_Site_C2, id_tail])
                    )
              → let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C2) = bond_Ct [] in
                  let (ch, id_Site_C1) = bond_Ch [] in
                    (
                      C [ch, ct],
                      let [s1, s2] = bond_AB [] in
                        (A [id_head, id_Site_C1, s1], B [s2, id_Site_C2, id_tail])
                    )
            let (ch, id_Site_C2) = bond_Ch [] in
                (C [ch, e_ABt []], R_dimerization [e_h [], e_C [], id_Site_C2, e_t []])
              : let (ch, id_Site_C2) = bond_Ch [] in
                (C [ch, e_ABt []], (A [e_h [], e_C [], e_t], B [e_h, id_Site_C2, e_t []]))
              → let (ch, id_Site_C2) = bond_Ch [] in
                (
                  C [ch, e_ABt []],
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, id_Site_C2, e_t []])
                )
            let (id_head, id_tail) = bond_AB [] in
                let (ch, id_Site_C2) = bond_Ch [] in
                  (C [ch, e_ABt []], R_dimerization [id_head, e_C [], id_Site_C2, id_tail])
              : let (id_head, id_tail) = bond_AB [] in
                let (ch, id_Site_C2) = bond_Ch [] in
                  (
                    C [ch, e_ABt []],
                    (A [id_head, e_C [], e_t], B [e_h, id_Site_C2, id_tail])
                  )
              → let (id_head, id_tail) = bond_AB [] in
                let (ch, id_Site_C2) = bond_Ch [] in
                  (
                    C [ch, e_ABt []],
                    let [s1, s2] = bond_AB [] in
                      (A [id_head, e_C [], s1], B [s2, id_Site_C2, id_tail])
                  )
            let (ct, id_Site_C1) = bond_Ct [] in
                let (ch, id_Site_C2) = bond_Ch [] in
                  (C [ch, ct], R_dimerization [e_h [], id_Site_C1, id_Site_C2, e_t []])
              : let (ct, id_Site_C1) = bond_Ct [] in
                let (ch, id_Site_C2) = bond_Ch [] in
                  (C [ch, ct], (A [e_h [], id_Site_C1, e_t], B [e_h, id_Site_C2, e_t []]))
              → let (ct, id_Site_C1) = bond_Ct [] in
                let (ch, id_Site_C2) = bond_Ch [] in
                  (
                    C [ch, ct],
                    let [s1, s2] = bond_AB [] in
                      (A [e_h [], id_Site_C1, s1], B [s2, id_Site_C2, e_t []])
                  )
            let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C1) = bond_Ct [] in
                  let (ch, id_Site_C2) = bond_Ch [] in
                    (C [ch, ct], R_dimerization [id_head, id_Site_C1, id_Site_C2, id_tail])
              : let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C1) = bond_Ct [] in
                  let (ch, id_Site_C2) = bond_Ch [] in
                    (
                      C [ch, ct],
                      (A [id_head, id_Site_C1, e_t], B [e_h, id_Site_C2, id_tail])
                    )
              → let (id_head, id_tail) = bond_AB [] in
                let (ct, id_Site_C1) = bond_Ct [] in
                  let (ch, id_Site_C2) = bond_Ch [] in
                    (
                      C [ch, ct],
                      let [s1, s2] = bond_AB [] in
                        (A [id_head, id_Site_C1, s1], B [s2, id_Site_C2, id_tail])
                    )
            let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  R_dimerization [e_h [], e_C [], e_C [], id_tail#1],
                  R_dimerization [id_head#2, e_C [], e_C [], e_t []]
                )
              : let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  (A [e_h [], e_C [], e_t], B [e_h, e_C [], id_tail#1]),
                  (A [id_head#2, e_C [], e_t], B [e_h, e_C [], e_t []])
                )
              → let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, e_C [], id_tail#1]),
                  let [s1, s2] = bond_AB [] in
                    (A [id_head#2, e_C [], s1], B [s2, e_C [], e_t []])
                )
            let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  R_dimerization [id_head#1, e_C [], e_C [], e_t []],
                  R_dimerization [e_h [], e_C [], e_C [], id_tail#2]
                )
              : let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  (A [id_head#1, e_C [], e_t], B [e_h, e_C [], e_t []]),
                  (A [e_h [], e_C [], e_t], B [e_h, e_C [], id_tail#2])
                )
              → let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  let [s1, s2] = bond_AB [] in
                    (A [id_head#1, e_C [], s1], B [s2, e_C [], e_t []]),
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, e_C [], id_tail#2])
                )
            let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    R_dimerization [id_head#1, e_C [], e_C [], id_tail#1],
                    R_dimerization [id_head#2, e_C [], e_C [], id_tail#2]
                  )
              : let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    (A [id_head#1, e_C [], e_t], B [e_h, e_C [], id_tail#1]),
                    (A [id_head#2, e_C [], e_t], B [e_h, e_C [], id_tail#2])
                  )
              → let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    let [s1, s2] = bond_AB [] in
                      (A [id_head#1, e_C [], s1], B [s2, e_C [], id_tail#1]),
                    let [s1, s2] = bond_AB [] in
                      (A [id_head#2, e_C [], s1], B [s2, e_C [], id_tail#2])
                  )
            let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  R_dimerization [e_h [], e_C [], e_C [], id_tail#1],
                  R_trimerization [id_head#2, e_t []]
                )
              : let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  (A [e_h [], e_C [], e_t], B [e_h, e_C [], id_tail#1]),
                  (
                    let [s1, s2] = bond [] in (A [id_head#2, e_C, s1], B [s2, e_C, e_t []]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  let [s1, s2] = bond_AB [] in
                    (A [e_h [], e_C [], s1], B [s2, e_C [], id_tail#1]),
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [id_head#2, ac, ab], B [ba, bc, e_t []], C [ca, cb])
                )
            let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  R_dimerization [id_head#1, e_C [], e_C [], e_t []],
                  R_trimerization [e_h [], id_tail#2]
                )
              : let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  (A [id_head#1, e_C [], e_t], B [e_h, e_C [], e_t []]),
                  (
                    let [s1, s2] = bond [] in (A [e_h [], e_C, s1], B [s2, e_C, id_tail#2]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  let [s1, s2] = bond_AB [] in
                    (A [id_head#1, e_C [], s1], B [s2, e_C [], e_t []]),
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [e_h [], ac, ab], B [ba, bc, id_tail#2], C [ca, cb])
                )
            let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    R_dimerization [id_head#1, e_C [], e_C [], id_tail#1],
                    R_trimerization [id_head#2, id_tail#2]
                  )
              : let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    (A [id_head#1, e_C [], e_t], B [e_h, e_C [], id_tail#1]),
                    (
                      let [s1, s2] = bond [] in
                        (A [id_head#2, e_C, s1], B [s2, e_C, id_tail#2]),
                      C [e_Ch, e_Ct]
                    )
                  )
              → let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    let [s1, s2] = bond_AB [] in
                      (A [id_head#1, e_C [], s1], B [s2, e_C [], id_tail#1]),
                    let [ac, ca] = bond_Ch [] in
                      let [bc, cb] = bond_Ct [] in
                        let [ab, ba] = bond_AB [] in
                          (A [id_head#2, ac, ab], B [ba, bc, id_tail#2], C [ca, cb])
                  )
            let (id_tail#1, id_head#2) = bond_AB [] in
                (R_trimerization [e_h [], id_tail#1], R_trimerization [id_head#2, e_t []])
              : let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  (
                    let [s1, s2] = bond [] in (A [e_h [], e_C, s1], B [s2, e_C, id_tail#1]),
                    C [e_Ch, e_Ct]
                  ),
                  (
                    let [s1, s2] = bond [] in (A [id_head#2, e_C, s1], B [s2, e_C, e_t []]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (id_tail#1, id_head#2) = bond_AB [] in
                (
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [e_h [], ac, ab], B [ba, bc, id_tail#1], C [ca, cb]),
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [id_head#2, ac, ab], B [ba, bc, e_t []], C [ca, cb])
                )
            let (id_head#1, id_tail#2) = bond_AB [] in
                (R_trimerization [id_head#1, e_t []], R_trimerization [e_h [], id_tail#2])
              : let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  (
                    let [s1, s2] = bond [] in (A [id_head#1, e_C, s1], B [s2, e_C, e_t []]),
                    C [e_Ch, e_Ct]
                  ),
                  (
                    let [s1, s2] = bond [] in (A [e_h [], e_C, s1], B [s2, e_C, id_tail#2]),
                    C [e_Ch, e_Ct]
                  )
                )
              → let (id_head#1, id_tail#2) = bond_AB [] in
                (
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [id_head#1, ac, ab], B [ba, bc, e_t []], C [ca, cb]),
                  let [ac, ca] = bond_Ch [] in
                    let [bc, cb] = bond_Ct [] in
                      let [ab, ba] = bond_AB [] in
                        (A [e_h [], ac, ab], B [ba, bc, id_tail#2], C [ca, cb])
                )
            let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    R_trimerization [id_head#1, id_tail#1],
                    R_trimerization [id_head#2, id_tail#2]
                  )
              : let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    (
                      let [s1, s2] = bond [] in
                        (A [id_head#1, e_C, s1], B [s2, e_C, id_tail#1]),
                      C [e_Ch, e_Ct]
                    ),
                    (
                      let [s1, s2] = bond [] in
                        (A [id_head#2, e_C, s1], B [s2, e_C, id_tail#2]),
                      C [e_Ch, e_Ct]
                    )
                  )
              → let (id_tail#1, id_head#2) = bond_AB [] in
                let (id_head#1, id_tail#2) = bond_AB [] in
                  (
                    let [ac, ca] = bond_Ch [] in
                      let [bc, cb] = bond_Ct [] in
                        let [ab, ba] = bond_AB [] in
                          (A [id_head#1, ac, ab], B [ba, bc, id_tail#1], C [ca, cb]),
                    let [ac, ca] = bond_Ch [] in
                      let [bc, cb] = bond_Ct [] in
                        let [ab, ba] = bond_AB [] in
                          (A [id_head#2, ac, ab], B [ba, bc, id_tail#2], C [ca, cb])
                  )"#]];
        transitions.assert_eq(&generator.transitions(2).join("\n")); // TODO: consider stronger typing to avoid explosion of transitions
    }

    #[test]
    fn toy_model_phospho_tyrosine() {
        let model = model::toy_model_phospho_tyrosine();
        let generator = NetGenerator::new(&model);

        let species = expect![[r#"
            A [e_sh2 []]
            C [u [e_xtyr []]]
            C [p [e_xtyr []]]
            let [s1, s2] = bond [] in (A [s1], C [p [s2]])"#]]; // @Evan, do you know why this complex is not generated? It appears in the RHS of the last transition.
        species.assert_eq(&generator.species(4).join("\n"));

        let transitions = expect![[r#"
            R_phosphorylation [] : A [u e_xtyr []] → A [p e_xtyr []]
            R_dimerization []
              : (A [e_sh2], C [p e_xtyr []])
              → let [s1, s2] = bond [] in (A [s1], C [p s2 []])"#]];
        transitions.assert_eq(&generator.transitions(2).join("\n")); // TODO: consider stronger typing to avoid explosion of transitions
    }
}
