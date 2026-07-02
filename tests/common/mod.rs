pub use rulett::{
    model::{Model, ModelDecl},
    netgen::NetGenerator,
    ob_tm::ObTm,
    surface,
    theory::{Signature, SignatureDecl},
    ty::Ty,
};

pub use expect_test::expect;

/// Function to merge two signatures
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
