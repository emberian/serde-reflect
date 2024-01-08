use serde_reflect::{Db, Reflect};
use proptest_derive::Arbitrary;

// this crate tries to exercise every feature of serde-reflect

// can derive for generic types!
#[derive(Reflect)]
struct Newtype<T>(T);

#[derive(Reflect)]

struct UnitLike;
#[derive(Reflect)]

struct TupleLike(String, Vec<u8>);
#[derive(Reflect)]

struct RealStruct {
    field: u8,
}

#[derive(Reflect)]
enum BigFinalType {
    UnitVariant,
    StructUnit(UnitLike),
    StructTuple(TupleLike),
    StructStruct(RealStruct),
    TupleVariant(u8, Newtype<()>),
    StructVariant { datum: String, wee_woo: usize },
}

#[test]
fn main() {
    let mut db = Db::new();
    BigFinalType::register(&mut db);
    proptest::proptest!(|(b: BigFinalType)| {
        let json_val = db.serialize(serde_json::value::Serializer, &b)?;
        let deser = db.deserialize(json_val);
        b == deser
    });
}
