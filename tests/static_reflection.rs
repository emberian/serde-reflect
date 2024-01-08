#![feature(const_type_id)]

use serde_reflect_derive::Reflect;
use proptest_derive::Arbitrary;
use serde_derive::{Deserialize, Serialize};

mod common;

use serde_reflect::*;
use std::{alloc::Layout, any::TypeId, borrow::Cow};

#[derive(PartialEq, Debug, Arbitrary, Serialize, Deserialize)]
struct ComplicatedThing;

#[derive(Reflect)]
struct Dynamic;

impl Reflect for ComplicatedThing {
    fn rust_type() -> StaticType {
        Cow::Borrowed(Self::RUST_TYPE)
    }
    fn register(db: &mut Db<'_, '_>) {
        db.register_const::<Self>();
    }
}
impl StaticReflect for ComplicatedThing {
    const RUST_TYPE: &'static ReflectedType<'static> = &ReflectedType {
        attrs: Cow::Borrowed(&[]),
        id: TypeId::of::<ComplicatedThing>(),
        layout: Layout::new::<ComplicatedThing>(),
        name: "ComplicatedThing",
        shape: DataShape::Struct(VariantData::Unit),
    };
}

#[test]
fn main() -> Result<(), anyhow::Error> {
    common::single_type::<ComplicatedThing>();
    let mut db = Db::new();
    // ensure we can insert a &'static Reflected<'static> and still use it dynamically
    db.insert(
        TypeId::of::<ComplicatedThing>(),
        Cow::Borrowed(ComplicatedThing::RUST_TYPE),
    );
    // ok, now we can insert a borrow of a stack local...
    let rfltyp = ComplicatedThing::RUST_TYPE.clone();
    db.insert(TypeId::of::<ComplicatedThing>(), Cow::Borrowed(&rfltyp));
    db.register_const::<ComplicatedThing>();
    db.register_type::<Dynamic>();
    db.register_const::<ComplicatedThing>();

    let json_val = db.serialize(serde_json::value::Serializer, &ComplicatedThing)?;
    let _: ComplicatedThing = db.deserialize(json_val)?;
    Ok(())
}
