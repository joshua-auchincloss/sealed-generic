#[derive(sealed_generic::SealedGeneric)]
#[define(
    types(ty = i32)
)]
pub struct BasicLt<'lt, T> {
    t: &'lt T,
}

#[test]
fn smoke() {
    let r = 0_i32;
    BasicLtI32 { t: &r };
}
