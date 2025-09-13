use sealed_generic::SealedGeneric;

#[derive(SealedGeneric)]
#[define(
    sealed,
    types(ty = i32),
)]
pub struct NoSlices<T: SealedSomeGeneric> {
    value: [T; 4],
}

fn main() {}
