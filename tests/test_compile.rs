use sealed_generic::SealedGeneric;

// derive "SealedGeneric"
#[derive(SealedGeneric)]
#[define(
    sealed,
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
struct Basic<T: SealedBasic> {
    #[allow(dead_code)]
    ty: T,
}

impl<T: SealedBasic> Basic<T> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        // the original generic type
        Basic::<i32> { ty: 1 };

        // the generated type for `i64`
        BasicI64 { ty: 1 };

        // the generated type for `u64`, where we #[derive(PartialEq, Eqs)]
        let u1 = BasicU64 { ty: 1 };
        let u2 = BasicU64 { ty: 1 };

        assert_eq!(u1, u2);

        // we can turn any of the derived types to / from the generic type
        let _: Basic<_> = u1.into();

        // the generated type for `i16`, where we
        // #[derive(serde::Serialize, serde::Deserialize)]
        // #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        let b = BasicI16 { ty: 1 };

        let ser = serde_json::to_string(&b).unwrap();
        assert_eq!(ser, "{\"TY\":1}");
    }
}
