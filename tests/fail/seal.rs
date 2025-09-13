use sealed_generic::SealedGeneric;

#[derive(SealedGeneric)]
#[define(
    sealed,
    types(ty = i32),
)]
pub struct SomeGeneric<T: SealedSomeGeneric> {
    value: T,
}

fn main() {
    SomeGeneric::<String> {
        value: "test".into(),
    };
}
