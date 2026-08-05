//! Networks derived from rule-based models.

use std::fmt;
use std::fs::File;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;

use super::{core::*, prelude::*};

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
#[derive(Clone, Debug, Default)]
pub struct Net {
    species: IndexSet<PatTm>,
    transitions: IndexMap<PatTm, (Vec<usize>, Vec<usize>)>,
}

impl Net {
    /// Constructs an empty net.
    pub fn new() -> Self {
        Self::default()
    }

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
    pub fn species_by_indexes(&self, idxs: &[usize]) -> Result<Vec<&PatTm>, usize> {
        idxs.iter().map(|&i| self.species.get_index(i).ok_or(i)).collect()
    }

    /// Gets the index of a species.
    pub fn index_of_species(&self, tm: &PatTm) -> Option<usize> {
        self.species.get_index_of(tm)
    }

    /// Gets the indexes of a list of species.
    pub fn indexes_of_species(&self, terms: &[PatTm]) -> Result<Vec<usize>, PatTm> {
        terms
            .iter()
            .map(|tm| self.species.get_index_of(tm).ok_or_else(|| tm.clone()))
            .collect()
    }

    /// Adds a species to the net, returning its index.
    pub fn add_species(&mut self, tm: PatTm) -> bool {
        self.species.insert(tm)
    }

    /// Adds a transition to the net.
    ///
    /// Returns an error with the offending pattern if any pattern in the source
    /// or target has not already been added as a species.
    pub fn add_transition(
        &mut self,
        tm: PatTm,
        src: &[PatTm],
        tgt: &[PatTm],
    ) -> Result<bool, PatTm> {
        let (src, tgt) = (self.indexes_of_species(src)?, self.indexes_of_species(tgt)?);
        Ok(self.transitions.insert(tm, (src, tgt)).is_none())
    }

    pub fn write_sbml<P: AsRef<Path>>(
        &self,
        path: P,
        initial_amounts: Option<HashMap<String, f64>>,
    ) -> Result<(), quick_xml::Error> {
        let model_name = path
            .as_ref()
            .file_stem() // Extracts the file stem as Option<&OsStr>
            .and_then(|s: &std::ffi::OsStr| s.to_str())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid or missing model name file stem",
                )
            })?;

        let species: Vec<String> = self.species().map(|tm| tm.to_string()).collect();
        let reactions: Vec<(&PatTm, &Vec<usize>, &Vec<usize>)> = self.transitions().collect();

        let file = File::create(&path)?;
        let mut writer = Writer::new_with_indent(BufWriter::new(file), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut sbml = BytesStart::new("sbml");
        sbml.push_attribute(("xmlns", "http://www.sbml.org/sbml/level3/version2/core"));
        sbml.push_attribute(("level", "3"));
        sbml.push_attribute(("version", "2"));
        writer.write_event(Event::Start(sbml))?;

        let mut model = BytesStart::new("model");
        model.push_attribute(("id", model_name));
        model.push_attribute(("name", model_name));
        writer.write_event(Event::Start(model))?;

        // --- compartments ---
        writer.write_event(Event::Start(BytesStart::new("listOfCompartments")))?;
        let mut compartment = BytesStart::new("compartment");
        compartment.push_attribute(("id", "default"));
        compartment.push_attribute(("size", "1"));
        compartment.push_attribute(("constant", "true"));
        writer.write_event(Event::Empty(compartment))?;
        writer.write_event(Event::End(BytesEnd::new("listOfCompartments")))?;

        // --- species ---
        writer.write_event(Event::Start(BytesStart::new("listOfSpecies")))?;
        for (i, name) in species.iter().enumerate() {
            let id = format!("s_{}", i);
            let ia = initial_amounts
                .as_ref()
                .and_then(|amounts| amounts.get(name))
                .copied()
                .unwrap_or(0.0)
                .to_string();
            let mut sp = BytesStart::new("species");
            sp.push_attribute(("id", id.as_str()));
            sp.push_attribute(("name", name.as_str()));
            sp.push_attribute(("compartment", "default"));
            sp.push_attribute(("initialAmount", ia.as_str()));
            sp.push_attribute(("hasOnlySubstanceUnits", "true"));
            // sp.push_attribute(("boundaryCondition", "false"));
            // sp.push_attribute(("constant", "false"));
            writer.write_event(Event::Empty(sp))?;
        }
        writer.write_event(Event::End(BytesEnd::new("listOfSpecies")))?;

        // --- global rate constants ---
        writer.write_event(Event::Start(BytesStart::new("listOfParameters")))?;
        for i in 0..reactions.len() {
            let id = format!("k_{}", i);
            let mut param = BytesStart::new("parameter");
            param.push_attribute(("id", id.as_str()));
            param.push_attribute(("value", "1")); // placeholder; edit as needed
            param.push_attribute(("constant", "true"));
            writer.write_event(Event::Empty(param))?;
        }
        writer.write_event(Event::End(BytesEnd::new("listOfParameters")))?;

        // --- reactions ---
        writer.write_event(Event::Start(BytesStart::new("listOfReactions")))?;
        for (i, (tm, reactants, products)) in reactions.iter().enumerate() {
            let reaction_id = format!("r_{}", i);
            let mut reaction = BytesStart::new("reaction");
            reaction.push_attribute(("id", reaction_id.as_str()));
            reaction.push_attribute(("name", tm.to_string().as_str()));
            reaction.push_attribute(("reversible", "false"));
            writer.write_event(Event::Start(reaction))?;

            write_species_reference_list(&mut writer, "listOfReactants", reactants)?;
            write_species_reference_list(&mut writer, "listOfProducts", products)?;
            write_mass_action_rate_law(&mut writer, i, reactants)?;

            writer.write_event(Event::End(BytesEnd::new("reaction")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("listOfReactions")))?;

        writer.write_event(Event::End(BytesEnd::new("model")))?;
        writer.write_event(Event::End(BytesEnd::new("sbml")))?;

        writer.into_inner().flush()?;
        Ok(())
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

/// Collapses a list of species indices into (index, stoichiometry) pairs,
/// sorted by index for stable output.
fn count_species(indices: &[usize]) -> Vec<(usize, usize)> {
    let mut counts: HashMap<usize, usize> = HashMap::new();
    for &idx in indices {
        *counts.entry(idx).or_insert(0) += 1;
    }
    let mut v: Vec<(usize, usize)> = counts.into_iter().collect();
    v.sort_by_key(|&(idx, _)| idx);
    v
}

fn write_species_reference_list<W: Write>(
    writer: &mut Writer<W>,
    tag: &str,
    indices: &[usize],
) -> Result<(), quick_xml::Error> {
    if indices.is_empty() {
        return Ok(());
    }
    writer.write_event(Event::Start(BytesStart::new(tag)))?;
    for (idx, count) in count_species(indices) {
        let species_id = format!("s_{}", idx);
        let stoich = count.to_string();
        let mut sr = BytesStart::new("speciesReference");
        sr.push_attribute(("species", species_id.as_str()));
        sr.push_attribute(("stoichiometry", stoich.as_str()));
        sr.push_attribute(("constant", "true"));
        writer.write_event(Event::Empty(sr))?;
    }
    writer.write_event(Event::End(BytesEnd::new(tag)))?;
    Ok(())
}

/// Writes a <kineticLaw> with mass-action kinetics: k * prod(reactant_i ^ stoichiometry_i)
fn write_mass_action_rate_law<W: Write>(
    writer: &mut Writer<W>,
    reaction_index: usize,
    reactants: &[usize],
) -> Result<(), quick_xml::Error> {
    let k_id = format!("k_{}", reaction_index);
    let counts = count_species(reactants);
    let num_factors = 1 + counts.len(); // rate constant + one term per distinct reactant

    writer.write_event(Event::Start(BytesStart::new("kineticLaw")))?;

    let mut math = BytesStart::new("math");
    math.push_attribute(("xmlns", "http://www.w3.org/1998/Math/MathML"));
    writer.write_event(Event::Start(math))?;

    if num_factors > 1 {
        writer.write_event(Event::Start(BytesStart::new("apply")))?;
        writer.write_event(Event::Empty(BytesStart::new("times")))?;
    }

    write_ci(writer, &k_id)?;

    for (sp_idx, count) in counts {
        let sp_id = format!("s_{}", sp_idx);
        if count > 1 {
            writer.write_event(Event::Start(BytesStart::new("apply")))?;
            writer.write_event(Event::Empty(BytesStart::new("power")))?;
            write_ci(writer, &sp_id)?;
            write_cn(writer, count)?;
            writer.write_event(Event::End(BytesEnd::new("apply")))?;
        } else {
            write_ci(writer, &sp_id)?;
        }
    }

    if num_factors > 1 {
        writer.write_event(Event::End(BytesEnd::new("apply")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("math")))?;
    writer.write_event(Event::End(BytesEnd::new("kineticLaw")))?;
    Ok(())
}

fn write_ci<W: Write>(writer: &mut Writer<W>, name: &str) -> Result<(), quick_xml::Error> {
    writer.write_event(Event::Start(BytesStart::new("ci")))?;
    writer.write_event(Event::Text(BytesText::new(name)))?;
    writer.write_event(Event::End(BytesEnd::new("ci")))?;
    Ok(())
}

fn write_cn<W: Write>(writer: &mut Writer<W>, value: usize) -> Result<(), quick_xml::Error> {
    let value_str = value.to_string();
    writer.write_event(Event::Start(BytesStart::new("cn")))?;
    writer.write_event(Event::Text(BytesText::new(&value_str)))?;
    writer.write_event(Event::End(BytesEnd::new("cn")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{super::model, super::netgen::NetGenerator};
    use insta;
    use std::fs;

    // Test output of toy_model_v2
    #[test]
    fn toy_model_v2() {
        use tempfile;

        let model = model::toy_model_v2();
        let generator = NetGenerator::new(&model);

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("toy_model_v2.xml");
        let initial_amounts = std::collections::HashMap::from([
            (String::from("A [unphos [], emptyA []]"), 1.0),
            (String::from("K []"), 1.0), // TODO: make this work for Int, too.
        ]);
        generator.net(2).write_sbml(&file_path, Some(initial_amounts)).unwrap();

        let actual_content =
            fs::read_to_string(&file_path).expect("Failed to read actual file output");

        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path("../tests/snapshots");
        settings.bind(|| {
            insta::assert_snapshot!("expected_report_snapshot", actual_content);
        });
    }
}
