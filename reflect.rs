#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
use serde_reflect::{Db, Reflect};
use proptest_derive::Arbitrary;
struct Newtype<T>(T);
const _: () = {
    extern crate serde_reflect as _reflect;
    extern crate core;
    extern crate memoffset;
    extern crate alloc;
    impl<T> _reflect::Reflect for Newtype<T>
    where
        Self: 'static,
    {
        fn rust_type() -> _reflect::StaticType {
            alloc::borrow::Cow::Owned(_reflect::ReflectedType {
                id: core::any::TypeId::of::<Self>(),
                name: "Newtype",
                layout: core::alloc::Layout::new::<Self>(),
                shape: _reflect::DataShape::Struct(_reflect::VariantData::Tuple(
                    alloc::borrow::Cow::Owned(<[_]>::into_vec(box [_reflect::TupleField {
                        offset: {
                            let uninit = ::memoffset::mem::MaybeUninit::<Self>::uninit();
                            let base_ptr: *const Self = uninit.as_ptr();
                            let field_ptr = {
                                #[allow(clippy::unneeded_field_pattern)]
                                let Self { 0: _, .. };
                                #[allow(unused_unsafe)]
                                unsafe {
                                    {
                                        &(*(base_ptr as *const Self)).0 as *const _
                                    }
                                }
                            };
                            (field_ptr as usize) - (base_ptr as usize)
                        },
                        type_id: core::any::TypeId::of::<Newtype>(),
                        attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                    }])),
                )),
                attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
            })
        }
        fn register(db: &mut _reflect::Db<'_, '_>) {
            db.register_type::<Self>();
            drop("oh well");
        }
    }
};
#[allow(non_upper_case_globals)]
const _IMPL_ARBITRARY_FOR_Newtype: () = {
    extern crate proptest as _proptest;
    impl<T: _proptest::arbitrary::Arbitrary> _proptest::arbitrary::Arbitrary for Newtype<T> {
        type Parameters = <T as _proptest::arbitrary::Arbitrary>::Parameters;
        type Strategy = _proptest::strategy::Map<
            (<T as _proptest::arbitrary::Arbitrary>::Strategy,),
            fn((T,)) -> Self,
        >;
        fn arbitrary_with(_top: Self::Parameters) -> Self::Strategy {
            {
                let param_0 = _top;
                _proptest::strategy::Strategy::prop_map(
                    (_proptest::arbitrary::any_with::<T>(param_0),),
                    |(tmp_0,)| Newtype { 0: tmp_0 },
                )
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl<T: ::core::fmt::Debug> ::core::fmt::Debug for Newtype<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            Newtype(ref __self_0_0) => {
                let mut debug_trait_builder = f.debug_tuple("Newtype");
                let _ = debug_trait_builder.field(&&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
struct UnitLike;
const _: () = {
    extern crate serde_reflect as _reflect;
    extern crate core;
    extern crate memoffset;
    extern crate alloc;
    impl _reflect::Reflect for UnitLike
    where
        Self: 'static,
    {
        fn rust_type() -> _reflect::StaticType {
            alloc::borrow::Cow::Owned(_reflect::ReflectedType {
                id: core::any::TypeId::of::<Self>(),
                name: "UnitLike",
                layout: core::alloc::Layout::new::<Self>(),
                shape: _reflect::DataShape::Struct(_reflect::VariantData::Unit),
                attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
            })
        }
        fn register(db: &mut _reflect::Db<'_, '_>) {
            db.register_type::<Self>();
            drop("oh well");
        }
    }
};
#[allow(non_upper_case_globals)]
const _IMPL_ARBITRARY_FOR_UnitLike: () = {
    extern crate proptest as _proptest;
    impl _proptest::arbitrary::Arbitrary for UnitLike {
        type Parameters = ();
        type Strategy = fn() -> Self;
        fn arbitrary_with(_top: Self::Parameters) -> Self::Strategy {
            || UnitLike {}
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for UnitLike {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            UnitLike => {
                let mut debug_trait_builder = f.debug_tuple("UnitLike");
                debug_trait_builder.finish()
            }
        }
    }
}
struct TupleLike(String, Vec<u8>);
const _: () = {
    extern crate serde_reflect as _reflect;
    extern crate core;
    extern crate memoffset;
    extern crate alloc;
    impl _reflect::Reflect for TupleLike
    where
        Self: 'static,
    {
        fn rust_type() -> _reflect::StaticType {
            alloc::borrow::Cow::Owned(_reflect::ReflectedType {
                id: core::any::TypeId::of::<Self>(),
                name: "TupleLike",
                layout: core::alloc::Layout::new::<Self>(),
                shape: _reflect::DataShape::Struct(_reflect::VariantData::Tuple(
                    alloc::borrow::Cow::Owned(<[_]>::into_vec(box [
                        _reflect::TupleField {
                            offset: {
                                let uninit = ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                let base_ptr: *const Self = uninit.as_ptr();
                                let field_ptr = {
                                    #[allow(clippy::unneeded_field_pattern)]
                                    let Self { 0: _, .. };
                                    #[allow(unused_unsafe)]
                                    unsafe {
                                        {
                                            &(*(base_ptr as *const Self)).0 as *const _
                                        }
                                    }
                                };
                                (field_ptr as usize) - (base_ptr as usize)
                            },
                            type_id: core::any::TypeId::of::<TupleLike>(),
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                        },
                        _reflect::TupleField {
                            offset: {
                                let uninit = ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                let base_ptr: *const Self = uninit.as_ptr();
                                let field_ptr = {
                                    #[allow(clippy::unneeded_field_pattern)]
                                    let Self { 1: _, .. };
                                    #[allow(unused_unsafe)]
                                    unsafe {
                                        {
                                            &(*(base_ptr as *const Self)).1 as *const _
                                        }
                                    }
                                };
                                (field_ptr as usize) - (base_ptr as usize)
                            },
                            type_id: core::any::TypeId::of::<TupleLike>(),
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                        },
                    ])),
                )),
                attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
            })
        }
        fn register(db: &mut _reflect::Db<'_, '_>) {
            db.register_type::<Self>();
            drop("oh well");
        }
    }
};
#[allow(non_upper_case_globals)]
const _IMPL_ARBITRARY_FOR_TupleLike: () = {
    extern crate proptest as _proptest;
    impl _proptest::arbitrary::Arbitrary for TupleLike {
        type Parameters = (
            <String as _proptest::arbitrary::Arbitrary>::Parameters,
            <Vec<u8> as _proptest::arbitrary::Arbitrary>::Parameters,
        );
        type Strategy = _proptest::strategy::Map<
            (
                <String as _proptest::arbitrary::Arbitrary>::Strategy,
                <Vec<u8> as _proptest::arbitrary::Arbitrary>::Strategy,
            ),
            fn((String, Vec<u8>)) -> Self,
        >;
        fn arbitrary_with(_top: Self::Parameters) -> Self::Strategy {
            {
                let (param_0, param_1) = _top;
                _proptest::strategy::Strategy::prop_map(
                    (
                        _proptest::arbitrary::any_with::<String>(param_0),
                        _proptest::arbitrary::any_with::<Vec<u8>>(param_1),
                    ),
                    |(tmp_0, tmp_1)| TupleLike { 0: tmp_0, 1: tmp_1 },
                )
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for TupleLike {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            TupleLike(ref __self_0_0, ref __self_0_1) => {
                let mut debug_trait_builder = f.debug_tuple("TupleLike");
                let _ = debug_trait_builder.field(&&(*__self_0_0));
                let _ = debug_trait_builder.field(&&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}
struct RealStruct {
    field: u8,
}
const _: () = {
    extern crate serde_reflect as _reflect;
    extern crate core;
    extern crate memoffset;
    extern crate alloc;
    impl _reflect::Reflect for RealStruct
    where
        Self: 'static,
    {
        fn rust_type() -> _reflect::StaticType {
            alloc::borrow::Cow::Owned(_reflect::ReflectedType {
                id: core::any::TypeId::of::<Self>(),
                name: "RealStruct",
                layout: core::alloc::Layout::new::<Self>(),
                shape: _reflect::DataShape::Struct(_reflect::VariantData::Fields {
                    labels_for_serde: {
                        const RealStruct_FIELDS_LABELS: &'static [&'static str] = &["field"];
                        RealStruct_FIELDS_LABELS
                    },
                    fields: alloc::borrow::Cow::Owned(<[_]>::into_vec(box [_reflect::Field {
                        offset: {
                            let uninit = ::memoffset::mem::MaybeUninit::<Self>::uninit();
                            let base_ptr: *const Self = uninit.as_ptr();
                            let field_ptr = {
                                #[allow(clippy::unneeded_field_pattern)]
                                let Self { field: _, .. };
                                #[allow(unused_unsafe)]
                                unsafe {
                                    {
                                        &(*(base_ptr as *const Self)).field as *const _
                                    }
                                }
                            };
                            (field_ptr as usize) - (base_ptr as usize)
                        },
                        type_id: core::any::TypeId::of::<RealStruct>(),
                        name: field,
                        attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                    }])),
                }),
                attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
            })
        }
        fn register(db: &mut _reflect::Db<'_, '_>) {
            db.register_type::<Self>();
            drop("oh well");
        }
    }
};
#[allow(non_upper_case_globals)]
const _IMPL_ARBITRARY_FOR_RealStruct: () = {
    extern crate proptest as _proptest;
    impl _proptest::arbitrary::Arbitrary for RealStruct {
        type Parameters = <u8 as _proptest::arbitrary::Arbitrary>::Parameters;
        type Strategy = _proptest::strategy::Map<
            (<u8 as _proptest::arbitrary::Arbitrary>::Strategy,),
            fn((u8,)) -> Self,
        >;
        fn arbitrary_with(_top: Self::Parameters) -> Self::Strategy {
            {
                let param_0 = _top;
                _proptest::strategy::Strategy::prop_map(
                    (_proptest::arbitrary::any_with::<u8>(param_0),),
                    |(tmp_0,)| RealStruct { field: tmp_0 },
                )
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for RealStruct {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            RealStruct {
                field: ref __self_0_0,
            } => {
                let mut debug_trait_builder = f.debug_struct("RealStruct");
                let _ = debug_trait_builder.field("field", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
enum BigFinalType {
    UnitVariant,
    StructUnit(UnitLike),
    StructTuple(TupleLike),
    StructStruct(RealStruct),
    TupleVariant(u8, Newtype<()>),
    StructVariant { datum: String, wee_woo: usize },
}
const _: () = {
    extern crate serde_reflect as _reflect;
    extern crate core;
    extern crate memoffset;
    extern crate alloc;
    impl _reflect::Reflect for BigFinalType
    where
        Self: 'static,
    {
        fn rust_type() -> _reflect::StaticType {
            alloc::borrow::Cow::Owned(_reflect::ReflectedType {
                id: core::any::TypeId::of::<Self>(),
                name: "BigFinalType",
                layout: core::alloc::Layout::new::<Self>(),
                shape: _reflect::DataShape::Enum {
                    variant_labels_for_serde: BigFinalType_VARIANTS_LABELS,
                    variants: alloc::borrow::Cow::Owned(<[_]>::into_vec(box [
                        _reflect::EnumArm {
                            label: "UnitVariant",
                            variant_index: 0usize,
                            discriminant: 0usize,
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                            variant: _reflect::VariantData::Unit,
                        },
                        _reflect::EnumArm {
                            label: "StructUnit",
                            variant_index: 1usize,
                            discriminant: 1usize,
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                            variant: _reflect::VariantData::Tuple(alloc::borrow::Cow::Owned(
                                <[_]>::into_vec(box [_reflect::TupleField {
                                    offset: {
                                        let uninit =
                                            ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                        let base_ptr: *const Self = uninit.as_ptr();
                                        let field_ptr = {
                                            #[allow(clippy::unneeded_field_pattern)]
                                            let Self { 0: _, .. };
                                            #[allow(unused_unsafe)]
                                            unsafe {
                                                {
                                                    &(*(base_ptr as *const Self)).0 as *const _
                                                }
                                            }
                                        };
                                        (field_ptr as usize) - (base_ptr as usize)
                                    },
                                    type_id: core::any::TypeId::of::<StructUnit>(),
                                    attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                }]),
                            )),
                        },
                        _reflect::EnumArm {
                            label: "StructTuple",
                            variant_index: 2usize,
                            discriminant: 2usize,
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                            variant: _reflect::VariantData::Tuple(alloc::borrow::Cow::Owned(
                                <[_]>::into_vec(box [_reflect::TupleField {
                                    offset: {
                                        let uninit =
                                            ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                        let base_ptr: *const Self = uninit.as_ptr();
                                        let field_ptr = {
                                            #[allow(clippy::unneeded_field_pattern)]
                                            let Self { 0: _, .. };
                                            #[allow(unused_unsafe)]
                                            unsafe {
                                                {
                                                    &(*(base_ptr as *const Self)).0 as *const _
                                                }
                                            }
                                        };
                                        (field_ptr as usize) - (base_ptr as usize)
                                    },
                                    type_id: core::any::TypeId::of::<StructTuple>(),
                                    attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                }]),
                            )),
                        },
                        _reflect::EnumArm {
                            label: "StructStruct",
                            variant_index: 3usize,
                            discriminant: 3usize,
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                            variant: _reflect::VariantData::Tuple(alloc::borrow::Cow::Owned(
                                <[_]>::into_vec(box [_reflect::TupleField {
                                    offset: {
                                        let uninit =
                                            ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                        let base_ptr: *const Self = uninit.as_ptr();
                                        let field_ptr = {
                                            #[allow(clippy::unneeded_field_pattern)]
                                            let Self { 0: _, .. };
                                            #[allow(unused_unsafe)]
                                            unsafe {
                                                {
                                                    &(*(base_ptr as *const Self)).0 as *const _
                                                }
                                            }
                                        };
                                        (field_ptr as usize) - (base_ptr as usize)
                                    },
                                    type_id: core::any::TypeId::of::<StructStruct>(),
                                    attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                }]),
                            )),
                        },
                        _reflect::EnumArm {
                            label: "TupleVariant",
                            variant_index: 4usize,
                            discriminant: 4usize,
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                            variant: _reflect::VariantData::Tuple(alloc::borrow::Cow::Owned(
                                <[_]>::into_vec(box [
                                    _reflect::TupleField {
                                        offset: {
                                            let uninit =
                                                ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                            let base_ptr: *const Self = uninit.as_ptr();
                                            let field_ptr = {
                                                #[allow(clippy::unneeded_field_pattern)]
                                                let Self { 0: _, .. };
                                                #[allow(unused_unsafe)]
                                                unsafe {
                                                    {
                                                        &(*(base_ptr as *const Self)).0 as *const _
                                                    }
                                                }
                                            };
                                            (field_ptr as usize) - (base_ptr as usize)
                                        },
                                        type_id: core::any::TypeId::of::<TupleVariant>(),
                                        attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                    },
                                    _reflect::TupleField {
                                        offset: {
                                            let uninit =
                                                ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                            let base_ptr: *const Self = uninit.as_ptr();
                                            let field_ptr = {
                                                #[allow(clippy::unneeded_field_pattern)]
                                                let Self { 1: _, .. };
                                                #[allow(unused_unsafe)]
                                                unsafe {
                                                    {
                                                        &(*(base_ptr as *const Self)).1 as *const _
                                                    }
                                                }
                                            };
                                            (field_ptr as usize) - (base_ptr as usize)
                                        },
                                        type_id: core::any::TypeId::of::<TupleVariant>(),
                                        attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                    },
                                ]),
                            )),
                        },
                        _reflect::EnumArm {
                            label: "StructVariant",
                            variant_index: 5usize,
                            discriminant: 5usize,
                            attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                            variant: _reflect::VariantData::Fields {
                                labels_for_serde: {
                                    const StructVariant_FIELDS_LABELS: &'static [&'static str] =
                                        &["datum", "wee_woo"];
                                    StructVariant_FIELDS_LABELS
                                },
                                fields: alloc::borrow::Cow::Owned(<[_]>::into_vec(box [
                                    _reflect::Field {
                                        offset: {
                                            let uninit =
                                                ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                            let base_ptr: *const Self = uninit.as_ptr();
                                            let field_ptr = {
                                                #[allow(clippy::unneeded_field_pattern)]
                                                let Self { datum: _, .. };
                                                #[allow(unused_unsafe)]
                                                unsafe {
                                                    {
                                                        &(*(base_ptr as *const Self)).datum
                                                            as *const _
                                                    }
                                                }
                                            };
                                            (field_ptr as usize) - (base_ptr as usize)
                                        },
                                        type_id: core::any::TypeId::of::<StructVariant>(),
                                        name: datum,
                                        attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                    },
                                    _reflect::Field {
                                        offset: {
                                            let uninit =
                                                ::memoffset::mem::MaybeUninit::<Self>::uninit();
                                            let base_ptr: *const Self = uninit.as_ptr();
                                            let field_ptr = {
                                                #[allow(clippy::unneeded_field_pattern)]
                                                let Self { wee_woo: _, .. };
                                                #[allow(unused_unsafe)]
                                                unsafe {
                                                    {
                                                        &(*(base_ptr as *const Self)).wee_woo
                                                            as *const _
                                                    }
                                                }
                                            };
                                            (field_ptr as usize) - (base_ptr as usize)
                                        },
                                        type_id: core::any::TypeId::of::<StructVariant>(),
                                        name: wee_woo,
                                        attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
                                    },
                                ])),
                            },
                        },
                    ])),
                },
                attrs: alloc::borrow::Cow::Owned(::alloc::vec::Vec::new()),
            })
        }
        fn register(db: &mut _reflect::Db<'_, '_>) {
            db.register_type::<Self>();
            drop("oh well");
        }
    }
};
#[allow(non_upper_case_globals)]
const _IMPL_ARBITRARY_FOR_BigFinalType: () = {
    extern crate proptest as _proptest;
    impl _proptest::arbitrary::Arbitrary for BigFinalType {
        type Parameters = (
            (<UnitLike as _proptest::arbitrary::Arbitrary>::Parameters),
            (<TupleLike as _proptest::arbitrary::Arbitrary>::Parameters),
            (<RealStruct as _proptest::arbitrary::Arbitrary>::Parameters),
            (
                <u8 as _proptest::arbitrary::Arbitrary>::Parameters,
                <Newtype<()> as _proptest::arbitrary::Arbitrary>::Parameters,
            ),
            (
                <String as _proptest::arbitrary::Arbitrary>::Parameters,
                <usize as _proptest::arbitrary::Arbitrary>::Parameters,
            ),
        );
        type Strategy = _proptest::strategy::TupleUnion<(
            (u32, ::std::sync::Arc<fn() -> Self>),
            (
                u32,
                ::std::sync::Arc<
                    _proptest::strategy::Map<
                        (<UnitLike as _proptest::arbitrary::Arbitrary>::Strategy,),
                        fn((UnitLike,)) -> Self,
                    >,
                >,
            ),
            (
                u32,
                ::std::sync::Arc<
                    _proptest::strategy::Map<
                        (<TupleLike as _proptest::arbitrary::Arbitrary>::Strategy,),
                        fn((TupleLike,)) -> Self,
                    >,
                >,
            ),
            (
                u32,
                ::std::sync::Arc<
                    _proptest::strategy::Map<
                        (<RealStruct as _proptest::arbitrary::Arbitrary>::Strategy,),
                        fn((RealStruct,)) -> Self,
                    >,
                >,
            ),
            (
                u32,
                ::std::sync::Arc<
                    _proptest::strategy::Map<
                        (
                            <u8 as _proptest::arbitrary::Arbitrary>::Strategy,
                            <Newtype<()> as _proptest::arbitrary::Arbitrary>::Strategy,
                        ),
                        fn((u8, Newtype<()>)) -> Self,
                    >,
                >,
            ),
            (
                u32,
                ::std::sync::Arc<
                    _proptest::strategy::Map<
                        (
                            <String as _proptest::arbitrary::Arbitrary>::Strategy,
                            <usize as _proptest::arbitrary::Arbitrary>::Strategy,
                        ),
                        fn((String, usize)) -> Self,
                    >,
                >,
            ),
        )>;
        fn arbitrary_with(_top: Self::Parameters) -> Self::Strategy {
            {
                let (param_0, param_1, param_2, param_3, param_4) = _top;
                _proptest::strategy::TupleUnion::new((
                    (1u32, ::std::sync::Arc::new(|| BigFinalType::UnitVariant {})),
                    (
                        1u32,
                        ::std::sync::Arc::new(_proptest::strategy::Strategy::prop_map(
                            (_proptest::arbitrary::any_with::<UnitLike>(param_0),),
                            |(tmp_0,)| BigFinalType::StructUnit { 0: tmp_0 },
                        )),
                    ),
                    (
                        1u32,
                        ::std::sync::Arc::new({
                            let param_0 = param_1;
                            _proptest::strategy::Strategy::prop_map(
                                (_proptest::arbitrary::any_with::<TupleLike>(param_0),),
                                |(tmp_0,)| BigFinalType::StructTuple { 0: tmp_0 },
                            )
                        }),
                    ),
                    (
                        1u32,
                        ::std::sync::Arc::new({
                            let param_0 = param_2;
                            _proptest::strategy::Strategy::prop_map(
                                (_proptest::arbitrary::any_with::<RealStruct>(param_0),),
                                |(tmp_0,)| BigFinalType::StructStruct { 0: tmp_0 },
                            )
                        }),
                    ),
                    (
                        1u32,
                        ::std::sync::Arc::new({
                            let (param_0, param_1) = param_3;
                            _proptest::strategy::Strategy::prop_map(
                                (
                                    _proptest::arbitrary::any_with::<u8>(param_0),
                                    _proptest::arbitrary::any_with::<Newtype<()>>(param_1),
                                ),
                                |(tmp_0, tmp_1)| BigFinalType::TupleVariant { 0: tmp_0, 1: tmp_1 },
                            )
                        }),
                    ),
                    (
                        1u32,
                        ::std::sync::Arc::new({
                            let (param_0, param_1) = param_4;
                            _proptest::strategy::Strategy::prop_map(
                                (
                                    _proptest::arbitrary::any_with::<String>(param_0),
                                    _proptest::arbitrary::any_with::<usize>(param_1),
                                ),
                                |(tmp_0, tmp_1)| BigFinalType::StructVariant {
                                    datum: tmp_0,
                                    wee_woo: tmp_1,
                                },
                            )
                        }),
                    ),
                ))
            }
        }
    }
};
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for BigFinalType {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match (&*self,) {
            (&BigFinalType::UnitVariant,) => {
                let mut debug_trait_builder = f.debug_tuple("UnitVariant");
                debug_trait_builder.finish()
            }
            (&BigFinalType::StructUnit(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("StructUnit");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
            (&BigFinalType::StructTuple(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("StructTuple");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
            (&BigFinalType::StructStruct(ref __self_0),) => {
                let mut debug_trait_builder = f.debug_tuple("StructStruct");
                let _ = debug_trait_builder.field(&&(*__self_0));
                debug_trait_builder.finish()
            }
            (&BigFinalType::TupleVariant(ref __self_0, ref __self_1),) => {
                let mut debug_trait_builder = f.debug_tuple("TupleVariant");
                let _ = debug_trait_builder.field(&&(*__self_0));
                let _ = debug_trait_builder.field(&&(*__self_1));
                debug_trait_builder.finish()
            }
            (&BigFinalType::StructVariant {
                datum: ref __self_0,
                wee_woo: ref __self_1,
            },) => {
                let mut debug_trait_builder = f.debug_struct("StructVariant");
                let _ = debug_trait_builder.field("datum", &&(*__self_0));
                let _ = debug_trait_builder.field("wee_woo", &&(*__self_1));
                debug_trait_builder.finish()
            }
        }
    }
}
extern crate test;
#[cfg(test)]
#[rustc_test_marker]
pub const main: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
        name: test::StaticTestName("main"),
        ignore: false,
        allow_fail: false,
        should_panic: test::ShouldPanic::No,
        test_type: test::TestType::IntegrationTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(main())),
};
#[allow(dead_code)]
fn main() {
    let mut db = Db::new();
    BigFinalType::register(&mut db);
    {
        let mut config = ::proptest::test_runner::Config::default().__sugar_to_owned();
        ::proptest::sugar::force_no_fork(&mut config);
        {
            config.source_file = Some("serde-reflect/tests/all_derive_features.rs");
            let mut runner = ::proptest::test_runner::TestRunner::new(config);
            let names = "b";
            match runner.run(
                &::proptest::strategy::Strategy::prop_map(
                    ::proptest::arbitrary::any::<BigFinalType>(),
                    |values| ::proptest::sugar::NamedArguments(names, values),
                ),
                |::proptest::sugar::NamedArguments(_, b)| {
                    let _: () = {
                        let json_val = db.serialize(serde_json::value::Serializer, &b)?;
                        let deser = db.deserialize(json_val);
                        b == deser
                    };
                    Ok(())
                },
            ) {
                Ok(_) => (),
                Err(e) => ::std::rt::begin_panic_fmt(&::core::fmt::Arguments::new_v1(
                    &["", "\n"],
                    &match (&e, &runner) {
                        (arg0, arg1) => [
                            ::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Display::fmt),
                            ::core::fmt::ArgumentV1::new(arg1, ::core::fmt::Display::fmt),
                        ],
                    },
                )),
            }
        };
    };
}
#[main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[&main])
}
