#![no_std]
//! Safe runtime reflection of Rust values.
//!
//! `serde_derive` is infamous for the sheer amount of code its liberal use can introduce
//! to a workspace. `serde_reflect` tries to combat that: avoid deriving `Serialize` and
//! `Deserialize` for most of your types, instead deriveing `Reflect`. The reflection
//! metadata for each type is reasonably small. Using the reflection metadata, a single
//! `Serialize`/`Deserialize` implementation can marshall any Rust type.
//!
//! This does not come for free, there are limitations on what types can be reflected:
//! `T: 'static` must hold so that `TypeId::of::<T>()` works. Without a type id,
use core::any::TypeId;

extern crate alloc;
use alloc::{borrow::Cow, collections::BTreeMap};

pub type StaticType = Cow<'static, ReflectedType<'static>>;
pub type DynamicType<'r> = Cow<'r, ReflectedType<'r>>;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate serde_reflect_derive;
#[doc(hidden)]
pub use memoffset;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use serde_reflect_derive::*;

mod de;
mod metadata;
mod ser;

pub use de::*;
pub use metadata::*;
pub use ser::*;

type DeserializeTrampoline =
    fn(&mut dyn erased_serde::Deserializer, &TypedOutputLocation) -> erased_serde::Result<()>;

/// Home of `serialize` and `deserialize`.
///
/// Known as the reflection database, this tracks the reflected types of all known
/// static Rust types, as well as how to serialize/deserialize.
///
/// # Lifetimes, the reflection database, and you
///
/// `serde-reflect` is designed so you can use ONLY compile-time generated
/// reflected types if you want, but it also allows runtime generated
/// reflected types. The only place these lifetimes appear in the public API is
/// `register_type`, but they leak into the definition of the type.
///
/// The `'r` lifetime represents (better yet, it is) how long the particular reflected type
/// references we are tracking are valid. Why not own the reflected types? Because then we
/// need to copy out the top-level metadata that could otherwise remain read-only
/// and shared.
///
/// The `'db` lifetime is how long the references _inside_ of the reflected type
/// are alive. For example, a reflected struct definition contains a reference to
/// the list of its fields.
#[derive(Default)]
pub struct Db<'r> {
    known_types: BTreeMap<TypeId, DynamicType<'r>>,
    deserialize_trampolines: BTreeMap<TypeId, DeserializeTrampoline>,
    serialize_vtables: BTreeMap<TypeId, *mut ()>,
}

/// Downcast the lifetime of a static type.
pub fn demote_static<'r>(x: StaticType) -> DynamicType<'r> {
    // SAFETY: this has _gotta_ be safe, right? I couldn't convince rustc
    // of that though. ReflectedType becomes invariant on its lifetime (and it
    // if very hard if not impossible to avoid that). The only references
    // it contains are Cow and they all have the same lifetime.
    unsafe {
        match x {
            Cow::Borrowed(b) => Cow::Borrowed(core::mem::transmute::<
                &'static ReflectedType<'static>,
                &'r ReflectedType<'r>,
            >(b)),
            Cow::Owned(o) => Cow::Owned(core::mem::transmute::<
                ReflectedType<'static>,
                ReflectedType<'r>,
            >(o)),
        }
    };
    unsafe { core::mem::transmute::<&'static ReflectedType<'static>, &'r ReflectedType<'r>>(x) }
}

impl<'db> Db<'db> {
    /// Make a new, empty reflection database.
    pub fn new() -> Self {
        Default::default()
    }

    /// Associate a `ReflectedType` with some runtime Rust type. Uses type id of `T`.
    pub fn register_type<T: Reflect>(&mut self) -> &mut Db<'db> {
        self.insert(TypeId::of::<T::Key>(), demote_static(T::rust_type()));
        self
    }

    /// Associate a `ReflectedType` with some const Rust type. Uses type id of `T`.
    pub fn register_const<T: StaticReflect>(&mut self) -> &mut Db<'db> {
        self.insert(
            TypeId::of::<T::Key>(),
            demote_static(Cow::Borrowed(T::RUST_TYPE)),
        );

        self
    }

    /// Old fashioned 100% runtime type registration. For when you're
    /// doing something that the derive can't help you with.
    pub fn insert(&mut self, id: TypeId, val: DynamicType<'db>) {
        self.known_types.insert(id, val);
    }

    /// When `T` is encountered during serialization, it will not be reflected into
    /// (even if it has a known type!) and instead the `Serialize`/`Deserialize`
    /// implementations will be used.
    ///
    /// `Default::default` is required to steal a trait object pointer without doing untoward.
    pub fn register_serde_leaf<T: Default + serde::Serialize + for<'de> serde::Deserialize<'de>>(
        &mut self,
    ) -> &mut Db<'db> {
        fn de<T: for<'b> serde::Deserialize<'b>>(
            d: &mut dyn erased_serde::Deserializer,
            dst: &TypedOutputLocation,
        ) -> Result<(), erased_serde::Error> {
            let x: T = T::deserialize(d)?;
            // SAFETY: if everything has gone right, this is the crucial line that the correctness
            // of the reflection system is to ensure.
            unsafe { dst.ptr.cast::<T>().write(x) };
            Ok(())
        }
        let typeid = TypeId::of::<T>();
        let vtable = unsafe {
            core::mem::transmute::<&dyn erased_serde::Serialize, TraitObject>(
                &Default::default() as &dyn erased_serde::Serialize
            )
            .vtable
        };
        self.serialize_vtables.insert(typeid, vtable);
        self.deserialize_trampolines.insert(typeid, de::<T>);
        self
    }

    fn type_layout(&self, id: TypeId) -> Result<DynamicType<'db>, alloc::string::String> {
        self.known_types
            .get(&id)
            .cloned()
            .ok_or_else(|| alloc::format!("reflection db missing type info for typeid {:?}", id))
    }
}

/// Trait for types which can ponder at runtime and produce a description of themselves.
///
/// There is a blanket impl for types which implement `StaticReflect`.
pub unsafe trait Reflect {
    /// `TypeId::of::<R::Key>()` is the database index for this type.
    type Key: 'static;
    fn rust_type() -> StaticType;
    /// Call this method to register all of the leaf Serialize/Deserialize and Reflect implementations
    /// that this type depends on existing.
    fn register(db: &mut Db<'_>);
}

/// Trait for types which have a static `ReflectedType` available.
///
/// There is a `#[derive(Reflect)]` available that you probably want instead.
pub trait StaticReflect: Reflect {
    const RUST_TYPE: &'static ReflectedType<'static>;
}

/// A destination in memory with a known eventual type.
///
/// These are the "places" that `Db::deserialize_in_place` can deserialize into.
/// Nothing about the type is assumed to be initialized until deserialization
/// is complete. ⚠️ If deserialization fails, any intermediate owned values will
/// be **leaked**. ⚠️
///
/// **SAFETY**: the type must always match the actual content of the memory!
/// The easiest way to ensure this is to only use the `From` impls that work on
/// references to any type that implements `Reflect` to construct locations.
pub struct TypedOutputLocation<'db, 'data> {
    typ: &'db ReflectedType<'db>,
    ptr: *mut u8,
    _data: core::marker::PhantomData<&'data ()>,
}

impl<'db, 'data> TypedOutputLocation<'db, 'data> {
    /// Prepare to deserialize a value of `typ` into `ptr`.
    ///
    /// # Safety
    /// If `ptr` doesn't point to a large-enough region,
    /// then eventually deserialization is going to write a bunch of data
    /// into a suspect location.
    pub unsafe fn new(typ: &DynamicType<'db>, ptr: *mut u8) -> Self {
        let _data = Default::default();
        TypedOutputLocation {
            typ: typ.as_ref(),
            ptr,
            _data,
        }
    }
}

/// A location in memory with a known reflected type.
///
/// These can be used with `Db::serialize_any`.
///
/// **SAFETY**: the type must always match the actual content of the memory!
/// The easiest way to ensure this is to only use the `From` impls that work on
/// references to any type that implements `Reflect` to construct locations.
pub struct TypedLocation<'db, 'data> {
    typ: &'db ReflectedType<'db>,
    ptr: *const u8,
    _data: core::marker::PhantomData<&'data ()>,
}

impl<'db, 'data> TypedLocation<'db, 'data> {
    /// Prepare to deserialize a value of `typ` into `ptr`.
    ///
    /// # Safety
    ///
    /// If `typ` doesn't actually describe the region of memory, laundry will be eaten.
    pub unsafe fn new<'z: 'db>(typ: &'z DynamicType<'db>, ptr: *const u8) -> Self {
        let _data = Default::default();
        TypedLocation {
            typ: typ.as_ref(),
            ptr,
            _data,
        }
    }
}

impl TypedLocation<'_, '_> {
    /// SAFETY: the location must be an enum.
    unsafe fn read_discriminant(&self) -> u16 {
        debug_assert!(matches!(self.typ.shape, DataShape::Enum { .. }));
        self.ptr.cast::<u16>().read()
    }
}

impl TypedOutputLocation<'_, '_> {
    /// SAFETY: the location must be an enum
    unsafe fn write_discriminant(&self, disc: u16) {
        self.ptr.cast::<u16>().write(disc)
    }
}

//#region Conversions

impl<'a, T: StaticReflect> From<&'a mut T> for TypedLocation<'static, 'a> {
    fn from(v: &'a mut T) -> Self {
        Self {
            typ: T::RUST_TYPE,
            ptr: v as *mut _ as *const _,
            _data: Default::default(),
        }
    }
}

impl<'a, T: StaticReflect> From<&'a mut T> for TypedOutputLocation<'static, 'a> {
    fn from(v: &'a mut T) -> Self {
        Self {
            typ: T::RUST_TYPE,
            ptr: v as *mut _ as *mut _,
            _data: Default::default(),
        }
    }
}

impl<'a, T: StaticReflect> From<&'a mut core::mem::MaybeUninit<T>>
    for TypedOutputLocation<'static, 'a>
{
    fn from(v: &'a mut core::mem::MaybeUninit<T>) -> Self {
        Self {
            typ: T::RUST_TYPE,
            ptr: v.as_mut_ptr().cast(),
            _data: Default::default(),
        }
    }
}

impl<'a, T: StaticReflect> From<&'a T> for TypedLocation<'static, 'a> {
    fn from(v: &'a T) -> Self {
        Self {
            typ: T::RUST_TYPE,
            ptr: v as *const _ as *const _,
            _data: Default::default(),
        }
    }
}

impl<'db, 'data> From<TypedOutputLocation<'db, 'data>> for TypedLocation<'db, 'data> {
    fn from(v: TypedOutputLocation<'db, 'data>) -> Self {
        Self {
            typ: v.typ,
            ptr: v.ptr as *const _,
            _data: Default::default(),
        }
    }
}

//#endregion

// the type in std is unstable. the layout isn't.
// https://doc.rust-lang.org/std/raw/struct.TraitObject.html
#[repr(C)]
pub(crate) struct TraitObject {
    pub(crate) data: *mut (),
    pub(crate) vtable: *mut (),
}
