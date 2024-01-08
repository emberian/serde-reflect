use crate::metadata::*;
use crate::{Db, TraitObject, TypedLocation};

use alloc::string::String;
use core::any::{Any, TypeId};
use serde::ser::Error as serError;
use serde::ser::{
    SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant,
};

impl<'db> Db<'db> {
    /// Serialize some value.
    ///
    /// The reflection database is consulted for the reflection type.
    pub fn serialize<S: serde::Serializer, T: Any>(
        &self,
        s: S,
        val: &T,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::{Error, Serialize};
        let rust_type = self
            .known_types
            .get(&TypeId::of::<T>())
            .ok_or_else(|| S::Error::custom("type missing from db, cannot serialize"))?;
        Serialize(
            self,
            TypedLocation {
                typ: rust_type,
                ptr: val as *const _ as *const _,
                _data: Default::default(),
            },
        )
        .serialize(s)
    }

    fn serialize_leaf<S: serde::Serializer>(
        &self,
        s: S,
        src: &TypedLocation<'_, '_>,
    ) -> Result<S::Ok, S::Error> {
        let vtable = *self.serialize_vtables.get(&src.typ.id).unwrap();
        let obj = unsafe {
            // SAFETY: the vtable is correct because we populated it correctly in register_leaf.
            core::mem::transmute::<TraitObject, &dyn erased_serde::Serialize>(TraitObject {
                vtable,
                data: src.ptr as *mut _,
            })
        };
        erased_serde::serialize(obj, s)
    }

    fn serialize_fields<I: IntoIterator<Item = &'db TupleField<'db>>, Err>(
        &self,
        mut f: impl FnMut(TypedLocation<'db, '_>) -> Result<(), Err>,
        fields_of: &TypedLocation<'db, '_>,
        fields: I,
        ty_err: &impl Fn(String) -> Err,
    ) -> Result<(), Err> {
        for &TupleField {
            offset, type_id, ..
        } in fields.into_iter()
        {
            let field_type = self.type_layout(type_id).map_err(ty_err)?;
            // SAFETY: TypedLocation contract
            unsafe {
                f(TypedLocation::new(&field_type, fields_of.ptr.add(offset)))?;
            }
        }
        Ok(())
    }

    fn serialize_named_fields<'data, I: IntoIterator<Item = &'db Field<'db>>, Err>(
        &self,
        mut f: impl FnMut(&'static str, TypedLocation<'db, 'data>) -> Result<(), Err>,
        fields_of: &'db TypedLocation<'db, 'data>,
        fields: I,
        ty_err: &impl Fn(String) -> Err,
    ) -> Result<(), Err> {
        for &Field {
            offset,
            type_id,
            name,
            ..
        } in fields.into_iter()
        {
            let field_type = self.type_layout(type_id).map_err(ty_err)?;
            // SAFETY: TypedLocation contract
            unsafe {
                f(
                    name,
                    TypedLocation::new(&field_type, fields_of.ptr.add(offset)),
                )?;
            }
        }
        Ok(())
    }
}

/// Wrapper for serializing a value from memory via reflection. You're better off using `Db::serialize`.
pub struct Serialize<'db, 'data>(&'db Db<'db>, TypedLocation<'db, 'data>);

impl serde::Serialize for Serialize<'_, '_> {
    fn serialize<S>(&self, dst: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Serialize(db, input) = self;
        match &input.typ.shape {
            DataShape::Leaf(tid) => db.serialize_leaf(dst, input),
            DataShape::Tuple(fields) => {
                let mut tup = dst.serialize_tuple(fields.len())?;
                db.serialize_fields(
                    |src| tup.serialize_element(&Serialize(db, src)),
                    input,
                    fields.iter(),
                    &serError::custom,
                )?;
                tup.end()
            }
            DataShape::Newtype(nested_type) => dst.serialize_newtype_struct(
                input.typ.name,
                &Serialize(
                    db,
                    // SAFETY: we uphold the contract by assuming that the database
                    unsafe {
                        TypedLocation::new(
                            &db.type_layout(*nested_type).map_err(serError::custom)?,
                            input.ptr,
                        )
                    },
                ),
            ),
            DataShape::Struct(VariantData::Unit) => dst.serialize_unit_struct(input.typ.name),
            DataShape::Struct(VariantData::Tuple(fields)) => {
                let mut tup = dst.serialize_tuple_struct(input.typ.name, fields.len())?;
                db.serialize_fields(
                    |src| tup.serialize_field(&Serialize(db, src)),
                    input,
                    fields.iter(),
                    &serError::custom,
                )?;
                tup.end()
            }
            DataShape::Struct(VariantData::Fields { fields, .. }) => {
                let mut struc = dst.serialize_struct(input.typ.name, fields.len())?;
                db.serialize_named_fields(
                    |name, src| struc.serialize_field(name, &Serialize(db, src)),
                    input,
                    fields.iter(),
                    &serError::custom,
                )?;
                struc.end()
            }
            DataShape::Enum { variants, .. } => {
                // SAFETY: we know we're looking at an enum.
                let tag = unsafe { input.read_discriminant() };
                let arm_idx = variants.binary_search_by_key(&tag, |arm| arm.discriminant);
                let arm = match arm_idx {
                    // SAFETY: we just got ix
                    Ok(ix) => variants.get(ix).unwrap(),
                    Err(_) => {
                        return Err(serError::custom(
                            "runtime discriminant not described in reflection db",
                        ))
                    }
                };
                match &arm.variant {
                    VariantData::Unit => dst.serialize_unit_variant(
                        input.typ.name,
                        arm.variant_index as u32,
                        arm.label,
                    ),
                    VariantData::Fields { fields, .. } => {
                        let mut struc = dst.serialize_struct_variant(
                            input.typ.name,
                            arm.variant_index as u32,
                            arm.label,
                            fields.len(),
                        )?;
                        db.serialize_named_fields(
                            |name, src| struc.serialize_field(name, &Serialize(db, src)),
                            input,
                            fields.iter(),
                            &serError::custom,
                        )?;
                        struc.end()
                    }
                    VariantData::Tuple(fields) => {
                        let mut tup = dst.serialize_tuple_variant(
                            input.typ.name,
                            arm.variant_index as u32,
                            arm.label,
                            fields.len(),
                        )?;
                        db.serialize_fields(
                            |src| tup.serialize_field(&Serialize(db, src)),
                            input,
                            fields.iter(),
                            &serError::custom,
                        )?;
                        tup.end()
                    }
                }
            }
            /*src.deserialize_enum(
                dst.typ.name,
                variant_labels_for_serde,
                OutVisitor(self, &dst),
            ),*/
            &DataShape::FixedArray(type_id, len) => {
                let typ = db.type_layout(type_id).map_err(serError::custom)?;
                let mut seq = dst.serialize_seq(Some(len))?;
                let stride = typ.layout.pad_to_align().size();
                for ix in 0..len {
                    seq.serialize_element(&Serialize(
                        db,
                        // SAFETY: ix is inbounds and TypedLocation contract
                        unsafe { TypedLocation::new(&typ, input.ptr.add(ix * stride).cast()) },
                    ))?;
                }
                seq.end()
            }
        }
    }
}
