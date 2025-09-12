use defined_generic::DefinedGeneric;

#[derive(DefinedGeneric)]
#[define(
    types(ty = i32),
    types(with(
        ty = i64,
    )),
    types(with(
        ty = u64,
        derives = PartialEq,
        derives = Eq,
        derives = Debug
    )),
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

mod test {
    use super::*;

    #[test]
    fn smoke() {
        Basic::<i32> { ty: 1 };
        BasicI64 { ty: 1 };

        let u1 = BasicU64 { ty: 1 };
        let u2 = BasicU64 { ty: 1 };

        assert_eq!(u1, u2);

        let _: Basic<_> = u1.into();
        let b = BasicI16 { ty: 1 };
        let ser = serde_json::to_string(&b).unwrap();
        assert_eq!(ser, "{\"TY\":1}");
    }
}
