use alloc::borrow::Cow;
use core::any::TypeId;

/// A named field, offset from the base of its containing type.
#[derive(Clone, Debug)]
pub struct Field<'db> {
    /// offset relative to the containing type
    pub offset: usize,
    pub name: &'static str,
    #[cfg(feature = "attrs")]
    pub attrs: Cow<'db, [Attr]>,
    pub shape: DataShape<'db>,
}

pub struct WithFields<'db, Hdr>(Arc<SliceWithHeader<Hdr, Field<'db>>>);

/// An unnamed field, such as occurs in tuples or tuple-structs etc.
#[derive(Clone, Debug)]
pub struct TupleField<'a> {
    /// offset relative to the containing type
    pub offset: usize,
    #[cfg(feature = "attrs")]
    pub attrs: Cow<'a, [Attr]>,
    pub shape: DataShape<'a>,
}

pub struct WithTupleFields<'db, Hdr>(Arc<SliceWithHeader<Hdr, TupleField<'db>>>);

/// An `#[attribute]`.
///
/// The structure is lightly recursive, such that an attribute is also the things
/// inside a list-like attribute eg `#[derive(Serialize, Deserialize)]` both `Serialize`
// and `Deserialize` are `Attr::Name`.
#[derive(Clone, Debug)]
pub enum Attr {
    // #[foo]
    Name(&'static str),
    // #[foo(...)]
    List(&'static str, &'static [&'static Attr]),
    // #[bar = ...]
    NameValue(&'static str, &'static RustPrimitive<'static>),
}

/// A primitive Rust value that can occur in an attribute.
#[derive(Clone, Debug)]
pub enum RustPrimitive<'a> {
    Str(&'a str),
    ByteStr(&'a [u8]),
    Byte(u8),
    Char(char),
    Int(i128),
    Float(f64),
    Bool(bool),
}

/// A Rust type, of some kind, with a unique `TypeId`.
#[derive(Clone, Debug)]
pub struct ReflectedType<'a> {
    pub id: TypeId,
    pub name: &'static str,
    pub layout: core::alloc::Layout,
    pub typ: ItemDeclaration<'a>,
    #[cfg(feature = "attrs")]
    pub attrs: Cow<'a, [Attr]>,
}

pub enum ItemDeclaration<'a> {
    /// `struct Foo(T);`
    Newtype(&'a ReflectedType<'a>),
    /// `struct Foo...`
    Struct(VariantData<'a>),
    /// `enum Foo { ... }`
    ///
    /// The offsets in the variant fields are relative to the enum base, ie,
    /// the addr of the discriminant.
    Enum {
        variant_labels_for_serde: &'static [&'static str],
        /// Sorted by discriminant (first key)
        variants: Cow<'a, [EnumArm<'a>]>,
    },
}

/// The sorts of data carriers in "items" types (structs and enums).
#[derive(Clone, Debug)]
pub enum VariantData<'a> {
    /// No fields
    Unit,
    /// Named fields, {foo: T, ...}
    Fields {
        labels_for_serde: &'static [&'static str],
        fields: Cow<'a, [Field<'a>]>,
    },
    /// Unnamed fields (T, U, ...)
    Tuple(Cow<'a, [TupleField<'a>]>),
}

/// An arm of an enum declaration, describing a variant.
#[derive(Clone, Debug)]
pub struct EnumArm<'a> {
    pub label: &'static str,
    pub variant_index: u16,
    pub discriminant: u16,
    #[cfg(feature = "attrs")]
    pub attrs: Cow<'a, [Attr]>,
    pub variant: VariantData<'a>,
}

/// The world of rust type representation, according to serde-reflect.
///
/// Unions are not supported. Intentionally compatible with the serde data model.
#[derive(Clone, Debug)]
pub enum DataShape<'a> {
    /// The reflection system will go no further with this type.
    ///
    /// For serde integration, leafs types are sought after
    /// in the reflection database when they are encountered.
    Leaf(TypeId),
    /// A small builtin rust type like i128 or char.
    Builtin(RustBuiltin),
    /// `[T; n]`
    FixedArray(TypeId, usize),
    /// `&[T]`
    Slice(Cow<'a, DataShape<'a>>),
    // TODO: if it is a fat pointer, what do i want to do about it?
    /// `&T`. ⚠️⚠️ might be a fat pointer! ⚠️⚠️
    Ref(Cow<'a, DataShape<'a>>),
    /// `(T, U, ...)`
    Tuple(Cow<'a, [TupleField<'a>]>),
}

pub enum RustBuiltin {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    U128,
    I128,
    BOOLIN,
    CHAR,
}
