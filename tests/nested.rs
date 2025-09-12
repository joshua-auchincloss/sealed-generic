use sealed_generic::SealedGeneric;

#[derive(SealedGeneric)]
#[define(types(ty = i64))]
struct WithNestedFields<T> {
    opt: Vec<Vec<Option<T>>>,
}

#[test]
fn smoke() {
    let _ = WithNestedFieldsI64 {
        opt: vec![vec![Some(1)]],
    };
}
