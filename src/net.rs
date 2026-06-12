//! Networks derived from rule-based models.

use std::fmt;

use super::{prelude::*, tm::*};

/// Petri net, aka reaction network, derived from a rule-based model.
///
/// # Definitions
///
/// *Species*. The species of the net are indecomposable closed patterns. A
/// **closed pattern** is a pattern with trivial interface. A pattern is
/// **indecomposable** if it cannot be expressed as a nontrivial product of
/// other patterns.
///
/// *Transitions*. The transitions of the net are indecomposable closed rules. A
/// **closed rule** is a (derived) rule with trivial interface. A rule is
/// **indecomposable** if it cannot be expressed as a nontrivial composite or
/// product of other rules.
///
/// # Data structure
///
/// Because the data type of a species is a complex data structure, namely a
/// [pattern term](PatTm), each species is assigned an integer index. Methods
/// are provided to convert between species and their indexes.
pub struct Net {
    species: IndexSet<PatTm>,
    transitions: IndexMap<PatTm, (Vec<usize>, Vec<usize>)>,
}

impl Net {
    /// Iterates over species of net.
    pub fn species(&self) -> impl Iterator<Item = &PatTm> {
        self.species.iter()
    }

    /// Iterates over transitions of net.
    pub fn transitions(&self) -> impl Iterator<Item = (&PatTm, &Vec<usize>, &Vec<usize>)> {
        self.transitions.iter().map(|(t, (src, tgt))| (t, src, tgt))
    }

    /// Gets a species by index.
    pub fn species_by_index(&self, index: usize) -> Option<&PatTm> {
        self.species.get_index(index)
    }

    /// Gets a list of species by their indexes.
    pub fn species_by_indexes(&self, idxs: &[usize]) -> Option<Vec<&PatTm>> {
        idxs.iter().map(|&i| self.species.get_index(i)).collect()
    }

    /// Gets the index of a species.
    pub fn index_of_species(&self, tm: &PatTm) -> Option<usize> {
        self.species.get_index_of(tm)
    }

    /// Gets the indexes of a list of species.
    pub fn indexes_of_species(&self, terms: &[PatTm]) -> Option<Vec<usize>> {
        terms.iter().map(|tm| self.species.get_index_of(tm)).collect()
    }

    /// Adds a species to the net, returning its index.
    pub fn add_species(&mut self, tm: PatTm) -> bool {
        self.species.insert(tm)
    }

    /// Adds a transition to the net.
    ///
    /// Fails, returning null, if any species in the source or target does not
    /// already belong to the net.
    pub fn add_transition(&mut self, tm: PatTm, src: &[PatTm], tgt: &[PatTm]) -> Option<bool> {
        let (src, tgt) = (self.indexes_of_species(src)?, self.indexes_of_species(tgt)?);
        Some(self.transitions.insert(tm, (src, tgt)).is_none())
    }
}

impl fmt::Display for Net {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "#/ species:")?;
        for tm in self.species() {
            render_doc(tm.to_doc(), f)?;
            writeln!(f)?;
        }
        writeln!(f, "#/ transitions:")?;
        for (tm, src, tgt) in self.transitions() {
            let src_doc = bracketed("[", "]", src.iter().map(|&i| self.species[i].to_doc()));
            let tgt_doc = bracketed("[", "]", tgt.iter().map(|&i| self.species[i].to_doc()));
            render_doc(mor_doc(tm.to_doc(), src_doc, tgt_doc), f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
