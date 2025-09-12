use sealed_generic::SealedGeneric;

#[derive(SealedGeneric)]
#[define(types(ty = i64))]
struct WithOpt<T> {
    normal: i32,
    opt: Option<T>,
}

#[test]
fn smoke() {
    let _ = WithOptI64 {
        normal: 0,
        opt: Some(0),
    };
}
