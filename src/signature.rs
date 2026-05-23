//! Signatures for rule-based models.

use super::{prelude::*, ty::*};

/// A declaration in the definition of a signature.
pub enum SignatureDecl {
    Sort(Name),
    Operation(Name, Ty, Ty),
}

/// Signature for a rule-based model.
///
/// A signature freely generates a theory. For now, a signature is essentially a
/// Petri net. It should become a Σ-net.
#[derive(Default)]
pub struct Signature {
    sorts: IndexSet<Name>,
    operations: IndexMap<Name, (Ty, Ty)>,
}

impl<T: IntoIterator<Item = SignatureDecl>> From<T> for Signature {
    fn from(iter: T) -> Self {
        let mut sig = Self::default();
        for decl in iter {
            match decl {
                SignatureDecl::Sort(name) => {
                    sig.sorts.insert(name);
                }
                SignatureDecl::Operation(name, dom, cod) => {
                    sig.operations.insert(name, (dom, cod));
                }
            }
        }
        sig
    }
}

impl Signature {
    /// Adds a sort with the given name to the signature.
    pub fn add_sort(&mut self, name: Name) {
        self.sorts.insert(name);
    }

    /// Adds an operation with given name to the signature.
    pub fn add_operation(&mut self, _name: Name, _dom: Ty, _cod: Ty) {
        todo!()
    }
}
