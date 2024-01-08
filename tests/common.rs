use serde_reflect::*;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

pub fn single_type<
    T: Arbitrary
        + Reflect
        + Serialize
        + for<'d> Deserialize<'d>
        + PartialEq
        + std::fmt::Debug
        + 'static,
>() {
    let mut db = Db::new();
    T::register(&mut db);
    proptest::proptest!(|(orig: T)| {
        let json_val = db.serialize(serde_json::value::Serializer, &orig)?;
        let deser: T = db.deserialize(json_val)?;
        prop_assert_eq!(orig, deser)
    })
}
