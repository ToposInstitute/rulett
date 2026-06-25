pub use rulett::{
    // core::*,
    model::*,
    // model::{Model, ModelDecl},
    // net::*,
    netgen::NetGenerator,
    ob_tm::ObTm,
    surface,
    surface::*,
    theory::{Signature, SignatureDecl},
    // ty::Ty,
    ty::*,
};

pub use expect_test::expect;

/// Merges multiple signatures.
pub fn merge_signatures(sigs: &[Signature]) -> Signature {
    let mut merged = Signature::new();
    for sig in sigs {
        for sort in sig.sorts() {
            if !merged.sorts().any(|s| s == sort) {
                merged.add_sort(sort).unwrap();
            }
        }
        for (op, dom, cod) in sig.operations() {
            if merged.interface(&op).is_none() {
                merged.add_operation(op, dom.clone(), cod.clone()).unwrap();
            }
        }
    }
    merged
}
