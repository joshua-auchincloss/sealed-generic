# sealed-generic

Sealed generic is a crate which offers sealed discriminants of a generic type.


## Use Case

The primary use case for this library are use cases where you may not be able to provide generics, or require a concrete type due to separate constraints.

```rust
use sealed_generic::SealedGeneric;

// derive "SealedGeneric"
#[derive(SealedGeneric)]
#[define(
    // #[define(types(ty = i32))] generates "BasicI32"
    types(ty = i32),
    // #[define(types(with(ty = i64)))] generates "BasicI32" with no additional options
    types(with(
        ty = i64,
    )),
    // define "BasicU64", and derive #[derive(PartialEq, Eq, Debug)]
    types(with(
        ty = u64,
        derives = PartialEq,
        derives = Eq,
        derives = Debug
    )),
    // define "BasicI16", and derive #[derive(serde::Deserialize, serde::Serialize)]
    // with #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    types(
        with(
            ty = i16,
            attr = "serde(rename_all = \"SCREAMING_SNAKE_CASE\")",
            derives = serde::Deserialize,
            derives = serde::Serialize,
        )
    )
)]
struct Basic<T> {
    ty: T,
}
```


## Sealing
While turned off by default, you may seal the generic types of the struct to the types generated from the derive macro.

Note: you must add `Sealed<name of struct>` as a type constraint.

```rust
use sealed_generic::SealedGeneric;


#[derive(SealedGeneric)]
#[define(
    sealed,
    types(ty = i32),
)]
pub struct SomeGeneric<T: SealedSomeGeneric> {
    value: T
}


fn main() {
    SomeGeneric<i32>{ ty: 0_i32 };
    // OK
    
    SomeGeneric<String>{ ty: "".into() };
    // Compiler Error >> `String` does not implement `SealedSomeGeneric`
}
```

## Constraints

- You may only use one generic field
- No lifetimes (yet)
- Cannot combine generic fields with slice literals