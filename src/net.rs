//! Networks derived from rule-based models.

use std::fmt;
use std::fs::File;
use std::io::Write;

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

    pub fn export_sbml(&self, file_name: &str) -> Result<(), String> {
        let model_name = file_name.split('.').next().unwrap_or(file_name).to_string();

        let file = File::create(file_name).map_err(|e| e.to_string())?;
        let mut writer = file;

        let species: Vec<String> = self.species().map(|tm| tm.to_string()).collect();
        let reactions: Vec<(&PatTm, &Vec<usize>, &Vec<usize>)> = self.transitions().collect();

        write_line(&mut writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        write_line(
            &mut writer,
            r#"<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">"#,
        )?;
        write_line(
            &mut writer,
            &format!(
                " <model id=\"{}\" name=\"{}\">",
                xml_escape(&model_name),
                xml_escape(&model_name)
            ),
        )?;
        write_line(&mut writer, " <listOfCompartments>")?;
        write_line(
            &mut writer,
            " <compartment id=\"default\" name=\"default\" size=\"1\" constant=\"true\"/>",
        )?;
        write_line(&mut writer, " </listOfCompartments>")?;
        write_line(&mut writer, " <listOfSpecies>")?;

        for (index, name) in species.iter().enumerate() {
            write_line(
                &mut writer,
                &format!(
                    " <species id=\"species_{}\" name=\"{}\" compartment=\"default\" initialAmount=\"1\" hasOnlySubstanceUnits=\"true\"/>",
                    index,
                    xml_escape(name)
                ),
            )?;
        }

        write_line(&mut writer, " </listOfSpecies>")?;
        write_line(&mut writer, " <listOfReactions>")?;

        for (index, (tm, src, tgt)) in reactions.iter().enumerate() {
            write_line(
                &mut writer,
                &format!(
                    " <reaction id=\"reaction_{}\" name=\"{}\" reversible=\"false\">",
                    index,
                    xml_escape(&tm.to_string())
                ),
            )?;
            write_line(&mut writer, " <listOfReactants>")?;
            for &src_idx in src.iter() {
                write_line(
                    &mut writer,
                    &format!(
                        " <speciesReference species=\"species_{}\" stoichiometry=\"1\" constant=\"true\"/>",
                        src_idx
                    ),
                )?;
            }
            write_line(&mut writer, " </listOfReactants>")?;
            write_line(&mut writer, " <listOfProducts>")?;
            for &tgt_idx in tgt.iter() {
                write_line(
                    &mut writer,
                    &format!(
                        " <speciesReference species=\"species_{}\" stoichiometry=\"1\" constant=\"true\"/>",
                        tgt_idx
                    ),
                )?;
            }
            write_line(&mut writer, " </listOfProducts>")?;

            let rate_constant_id = format!("k{}", index);
            let math = build_mass_action_math(&rate_constant_id, &src);
            write_line(&mut writer, " <kineticLaw>")?;
            write_line(&mut writer, &format!(" {}", math))?;
            write_line(&mut writer, " </kineticLaw>")?;
            write_line(&mut writer, " </reaction>")?;
        }

        write_line(&mut writer, " </listOfReactions>")?;
        write_line(&mut writer, " </model>")?;
        write_line(&mut writer, "</sbml>")?;
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

fn write_line(writer: &mut impl Write, line: &str) -> Result<(), String> {
    writer.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    writer.write_all(b"\n").map_err(|e| e.to_string())
}

fn build_mass_action_math(rate_constant_id: &str, reactant_idxs: &[usize]) -> String {
    let mut math = String::new();
    math.push_str(r#"<math xmlns="http://www.w3.org/1998/Math/MathML">"#);
    math.push_str("<apply><times/>");
    math.push_str(&format!("<ci>{}</ci>", xml_escape(rate_constant_id)));
    for reactant in reactant_idxs.iter() {
        math.push_str(&format!("<ci>species_{}</ci>", reactant));
    }
    math.push_str("</apply>");
    math.push_str("</math>");
    math
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
}

#[cfg(test)]
mod tests {
    use super::{super::model, super::netgen::NetGenerator};
    use expect_test::expect;
    use std::fs;

    // Test output of toy_model_v1
    #[test]
    fn toy_model_v1() {
        let model = model::toy_model_v1();
        let generator = NetGenerator::new(&model);
        // let species = expect![[r#"
        // A [unphos [], empty []]
        // A [phos [], empty []]
        // B [empty []]
        // K []
        // let bond [] in (A [unphos [], 0.0], A [unphos [], 0.1])
        // let bond [] in (A [unphos [], 0.0], A [phos [], 0.1])
        // let bond [] in (A [phos [], 0.0], A [unphos [], 0.1])
        // let bond [] in (A [phos [], 0.0], A [phos [], 0.1])
        // let bond [] in (A [unphos [], 0.0], B [0.1])
        // let bond [] in (A [phos [], 0.0], B [0.1])
        // let bond [] in (B [0.0], B [0.1])"#]];
        // species.assert_eq(&generator.species(2).join("\n")); // method not found in `impl Iterator<Item = core::tm::PatTm>

        // "tests/fixtures/toy_model_v1.xml", but OS independent version
        let file_path = "tests/fixtures/toy_model_v1.xml";
        &generator.net(2).export_sbml(file_path).unwrap();

        // 2. Read the content of the generated file
        let actual_content =
            fs::read_to_string(&file_path).expect("Failed to read actual file output");

        // 3. Compare it against a snapshot reference file
        insta::assert_snapshot!("expected_report_snapshot", actual_content);
    }
}
