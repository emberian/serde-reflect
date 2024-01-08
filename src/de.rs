use crate::metadata::*;
use crate::{Db, TypedOutputLocation};

use alloc::{borrow::Cow, string::ToString};

use core::any::{Any, TypeId};
use erased_serde::Error;
use serde::de::Error as deError;

impl<'db> Db<'db> {
    /// Deserialize some value.
    ///
    /// The reflection database is consulted for the reflection type.
    pub fn deserialize<'de, T: Any, D>(
        &self,
        src: D,
    ) -> Result<T, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::DeserializeSeed;

        let rust_type = self
            .known_types
            .get(&TypeId::of::<T>())
            .ok_or_else(|| deError::custom("type missing from db, cannot deserialize"))?;
        let mut uninit = core::mem::MaybeUninit::uninit();
        Deserialize(
            self,
            TypedOutputLocation {
                ptr: uninit.as_mut_ptr() as *mut u8,
                typ: rust_type,
                _data: Default::default(),
            },
        )
        .deserialize(src)?;
        Ok(unsafe { uninit.assume_init() })
    }

    fn deserialize_leaf<'de, D>(
        &self,
        d: D,
        dst: &TypedOutputLocation<'db, '_>,
    ) -> Result<(), Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.deserialize_trampolines
            .get(&dst.typ.id)
            .ok_or_else(|| <Error as deError>::custom("leaf type missing from reflection db"))?(
            &mut erased_serde::Deserializer::erase(d),
            dst,
        )
        .map_err(|e| <Error as deError>::custom(e.to_string()))
    }
}
/// Wrapper for deserializing a value via reflection. You're better off using `Db::deserialize`.
pub struct Deserialize<'db, 'data>(&'db Db<'db>, TypedOutputLocation<'db, 'data>);

impl<'db, 'data, 'de> serde::de::DeserializeSeed<'de> for Deserialize<'db, 'data> {
    type Value = ();
    fn deserialize<D>(self, src: D) -> Result<(), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let db = self.0;
        let dst = self.1;
        match &dst.typ.shape {
            DataShape::Leaf(id) => db
                .deserialize_leaf(src, &dst)
                .map_err(|e| serde::de::Error::custom(e.to_string())),
            DataShape::Tuple(fields) => src.deserialize_tuple(fields.len(), TupleVisitor(db, &dst)),
            DataShape::Newtype(_) => {
                src.deserialize_newtype_struct(dst.typ.name, NewtypeVisitor(db, &dst))
            }
            DataShape::Struct(VariantData::Unit) => {
                src.deserialize_unit_struct(dst.typ.name, UnitVisitor(db, &dst))
            }
            DataShape::Struct(VariantData::Tuple(fields)) => {
                src.deserialize_tuple_struct(dst.typ.name, fields.len(), TupleVisitor(db, &dst))
            }
            DataShape::Struct(VariantData::Fields {
                labels_for_serde, ..
            }) => src.deserialize_struct(dst.typ.name, labels_for_serde, FieldsVisitor(db, &dst)),
            DataShape::Enum {
                variant_labels_for_serde,
                ..
            } => src.deserialize_enum(
                dst.typ.name,
                variant_labels_for_serde,
                EnumVisitor(db, &dst),
            ),
            DataShape::FixedArray(_, len) => src.deserialize_tuple(*len, TupleVisitor(db, &dst)),
        }
    }
}

struct TupleVisitor<'db, 'data, 'visitor>(&'db Db<'db>, &'visitor TypedOutputLocation<'db, 'data>);
impl<'r, 'db, 'de, 'a, 'b> serde::de::Visitor<'de> for TupleVisitor<'_, '_, '_> {
    type Value = ();

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let TupleVisitor(db, dst) = self;
        match &dst.typ.shape {
            DataShape::Leaf(id) => todo!(),
            &DataShape::FixedArray(type_id, len) => {
                let typ = db.type_layout(type_id).map_err(deError::custom)?;
                let stride = typ.layout.pad_to_align().size();
                for ix in 0..len {
                    seq.next_element_seed(Deserialize(self.0, unsafe {
                        // SAFETY: correctness of reflection data
                        TypedOutputLocation::new(&typ, dst.ptr.add(ix * stride).cast())
                    }))?;
                }
            }
            DataShape::Tuple(fields) | DataShape::Struct(VariantData::Tuple(fields)) => {
                for &TupleField {
                    type_id, offset, ..
                } in fields.iter()
                {
                    seq.next_element_seed(Deserialize(self.0, unsafe {
                        // SAFETY: correctness of reflection data
                        TypedOutputLocation::new(
                            &db.type_layout(type_id).map_err(deError::custom)?,
                            dst.ptr.add(offset).cast(),
                        )
                    }))?;
                }
            }
            DataShape::Struct(VariantData::Fields { fields, .. }) => {
                for &Field {
                    type_id, offset, ..
                } in fields.iter()
                {
                    seq.next_element_seed(Deserialize(self.0, unsafe {
                        // SAFETY: correctness of reflection data
                        TypedOutputLocation::new(
                            &db.type_layout(type_id).map_err(deError::custom)?,
                            dst.ptr.add(offset).cast(),
                        )
                    }))?;
                }
            }
            _ => return Err(deError::custom("unexpected shape when visiting sequence")),
        }
        Ok(())
    }

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "a sequence to fill the fields of {:?}", self.1.typ)
    }
}

struct UnitVisitor<'db, 'data, 'visitor>(&'db Db<'db>, &'visitor TypedOutputLocation<'db, 'data>);
impl<'de> serde::de::Visitor<'de> for UnitVisitor<'_, '_, '_> {
    type Value = ();

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: deError,
    {
        unsafe {
            self.1.ptr.cast::<()>().write(());
        }
        Ok(())
    }

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "a unit to fill {:?}", self.1.typ)
    }
}

struct FieldIx<'db>(&'static [&'static str], &'db Cow<'db, [Field<'db>]>);
impl<'de> serde::de::Visitor<'de> for FieldIx<'_> {
    type Value = usize;
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: deError,
    {
        if (v as usize) < self.1.len() {
            Ok(v as usize)
        } else {
            Err(E::unknown_field(&v.to_string(), self.0))
        }
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: deError,
    {
        // TODO: faster scan
        for (ix, f) in self.1.iter().enumerate() {
            if f.name == v {
                return Ok(ix);
            }
        }
        Err(E::invalid_value(
            serde::de::Unexpected::Str(v),
            &"a string that is one of the field labels",
        ))
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: deError,
    {
        for (ix, f) in self.1.iter().enumerate() {
            if f.name.as_bytes() == v {
                return Ok(ix);
            }
        }
        Err(E::unknown_field(
            core::str::from_utf8(v).unwrap_or("<non-utf8 fieldname>"),
            self.0,
        ))
    }
    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("a struct field")
    }
}
impl<'de> serde::de::DeserializeSeed<'de> for FieldIx<'_> {
    type Value = usize;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}
struct VariantIx<'f, 'e, 'a>(&'static [&'static str], &'f Cow<'e, [EnumArm<'a>]>);
impl<'de> serde::de::Visitor<'de> for VariantIx<'_, '_, '_> {
    type Value = usize;
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: deError,
    {
        if (v as usize) < self.1.len() {
            Ok(v as usize)
        } else {
            Err(E::unknown_variant(&v.to_string(), self.0))
        }
    }
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: deError,
    {
        // TODO: faster scan
        for (ix, f) in self.1.iter().enumerate() {
            if f.label == v {
                return Ok(ix);
            }
        }
        Err(E::unknown_variant(v, self.0))
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: deError,
    {
        for (ix, f) in self.1.iter().enumerate() {
            if f.label.as_bytes() == v {
                return Ok(ix);
            }
        }
        Err(E::unknown_field(
            core::str::from_utf8(v).unwrap_or("<non-utf8 fieldname>"),
            self.0,
        ))
    }
    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("a struct field")
    }
}
impl<'de> serde::de::DeserializeSeed<'de> for VariantIx<'_, '_, '_> {
    type Value = usize;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}
struct FieldsVisitor<'db, 'data, 'visitor>(&'db Db<'db>, &'visitor TypedOutputLocation<'db, 'data>);
impl<'de> serde::de::Visitor<'de> for FieldsVisitor<'_, '_, '_> {
    type Value = ();

    fn visit_map<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let FieldsVisitor(db, dst) = self;
        match &dst.typ.shape {
            DataShape::Struct(VariantData::Fields {
                fields,
                labels_for_serde,
            }) => {
                while let Some(ix) = seq.next_key_seed(FieldIx(labels_for_serde, fields))? {
                    seq.next_value_seed(Deserialize(
                        self.0,
                        // SAFETY: correctness of reflection data
                        unsafe {
                            TypedOutputLocation::new(
                                &db.type_layout(fields[ix].type_id)
                                    .map_err(deError::custom)?,
                                dst.ptr.add(fields[ix].offset).cast(),
                            )
                        },
                    ))?;
                }
            }
            _ => return Err(deError::custom("unexpected shape when visiting sequence")),
        }
        Ok(())
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        TupleVisitor(self.0, self.1).visit_seq(seq)
    }

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "a sequence to fill the fields of {:?}", self.1.typ)
    }
}

struct EnumVisitor<'db, 'data, 'visitor>(&'db Db<'db>, &'visitor TypedOutputLocation<'db, 'data>);
impl<'de> serde::de::Visitor<'de> for EnumVisitor<'_, '_, '_> {
    type Value = ();
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        use serde::de::VariantAccess;
        match &self.1.typ.shape {
            DataShape::Enum {
                variant_labels_for_serde,
                variants,
            } => match data.variant_seed(VariantIx(variant_labels_for_serde, variants)) {
                Ok((ix, variant)) => {
                    let arm = &variants[ix];
                    // SAFETY: TypedOutputLocation contract upheld, correctness of metadata.
                    unsafe {
                        self.1.write_discriminant(arm.discriminant);
                    }
                    match &arm.variant {
                        VariantData::Unit => variant.unit_variant(),
                        VariantData::Fields {
                            labels_for_serde, ..
                        } => {
                            variant.struct_variant(labels_for_serde, FieldsVisitor(self.0, self.1))
                        }
                        VariantData::Tuple(fields) => {
                            // SAFETY: we change the runtime type here so that the tuple visitor code isn't duplicated, but
                            // now that we've verified the discriminant, the field metadata is accurate when viewed as
                            // a struct.
                            unsafe {
                                variant.tuple_variant(
                                    fields.len(),
                                    TupleVisitor(
                                        self.0,
                                        &TypedOutputLocation::new(
                                            &Cow::Borrowed(&ReflectedType {
                                                typ: DataShape::Tuple(fields.clone()),
                                                ..self.1.typ.clone()
                                            }),
                                            self.1.ptr,
                                        ),
                                    ),
                                )
                            }
                        }
                    }
                }
                Err(e) => Err(e),
            },
            _shape => Err(deError::custom(
                "visited an enum when not expecting an enum",
            )),
        }
    }

    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("an enum")
    }
}

struct NewtypeVisitor<'db, 'data, 'visitor>(
    &'db Db<'db>,
    &'visitor TypedOutputLocation<'db, 'data>,
);

impl<'de> serde::de::Visitor<'de> for NewtypeVisitor<'_, '_, '_> {
    type Value = ();

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::DeserializeSeed;
        match self.1.typ.shape {
            DataShape::Newtype(id) => Deserialize(
                self.0,
                // SAFETY: TypedOutputLocation contract
                unsafe {
                    TypedOutputLocation::new(
                        &self.0.type_layout(id).map_err(deError::custom)?,
                        self.1.ptr,
                    )
                },
            )
            .deserialize(deserializer),

            _ => Err(deError::custom("visit and shape disagree")),
        }
    }
    fn expecting(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("newtype struct")
    }
}
