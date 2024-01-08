//! Automatically derive reflection metadata with `#[derive(Reflect)]`

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
extern crate alloc;

use syn::{Data, DataEnum, DataStruct, Fields, Ident};

// all the state and methods for doing a derive on a single item
struct DeriveReflect<'a> {
    consts: Vec<TokenStream>,
    nesting: Option<String>,
    // the parent stack tracks the field path through an item
    parent_stack: Vec<Ident>,
    most_recent_discriminant_expr: Option<(usize, TokenStream)>,
    nightly_const: bool,
    generics: &'a syn::Generics,
    seen_types: Vec<&'a syn::Type>,
}

fn primitive(p: &syn::Lit) -> TokenStream {
    match p {
        syn::Lit::Str(s) => quote! {_reflect::RustPrimitive::Str(#s)},
        syn::Lit::ByteStr(bs) => quote! {_reflect::RustPrimitive::ByteStr(#bs)},
        syn::Lit::Byte(b) => quote! {_reflect::RustPrimitive::Byte(#b)},
        syn::Lit::Char(c) => quote! {_reflect::RustPrimitive::Char(#c)},
        syn::Lit::Int(i) => {
            // don't format the LitInt directly because it might be eg `4u32` which isn't what we want
            let i = i
                .base10_parse::<i128>()
                .expect("syntax error after parsing?");
            quote! {_reflect::RustPrimitive::Int(#i) }
        }
        syn::Lit::Float(f) => {
            // similar to above, this might be 0x123f32 which is not what we're after
            let f = f
                .base10_parse::<f64>()
                .expect("syntax error after parsing?");
            quote! {_reflect::RustPrimitive::Float(#f) }
        }
        syn::Lit::Bool(b) => quote! {_reflect::RustPrimitive::Bool(#b)},
        syn::Lit::Verbatim(v) => panic!("not really sure what literal this is: {:?}", v),
    }
}

impl<'a> DeriveReflect<'a> {
    fn parent(&self) -> Ident {
        format_ident!(
            "{}",
            self.parent_stack.iter().map(|x| x.to_string()).join("_")
        )
    }

    fn nested<T>(&mut self, ix: usize, f: impl FnOnce(&mut Self) -> T) -> T {
        let old_nesting = self.nesting.replace(
            self.nesting
                .as_ref()
                .map_or(ix.to_string(), |n| format!("{}_{}", n, ix)),
        );
        let res = f(self);
        self.nesting = old_nesting;
        res
    }

    fn meta(&mut self, ix: usize, m: &syn::Meta) -> TokenStream {
        match m {
            syn::Meta::Path(p) => {
                let p = p.to_token_stream().to_string();
                quote! { Attr::Name(#p)  }
            }
            syn::Meta::List(ml) => {
                let nesteds = ml
                    .nested
                    .iter()
                    .map(|nested| match nested {
                        syn::NestedMeta::Meta(m) => self.nested(ix, |me| me.meta(0, m)),
                        syn::NestedMeta::Lit(l) => l.to_token_stream(),
                    })
                    .collect::<Vec<_>>();
                let path = ml.path.to_token_stream().to_string();
                let nesteds_ident = format_ident!(
                    "{}_META_{}_{}",
                    self.parent(),
                    ix,
                    self.nesting.as_ref().unwrap_or(&String::new())
                );
                self.consts.push(quote!{const #nesteds_ident : &'static [_reflect::Attr<'static>] = &[#(#nesteds),*]; });
                let list = self.list_reference(nesteds_ident, nesteds);
                quote! { _reflect::Attr::List(#path, #list) }
            }
            syn::Meta::NameValue(mnv) => {
                let path = mnv.path.to_token_stream().to_string();
                let lit = primitive(&mnv.lit);
                quote! { _reflect::Attr::NameValue(#path, #lit) }
            }
        }
    }
    fn list_reference(&self, id: Ident, elts: Vec<TokenStream>) -> TokenStream {
        if self.nightly_const {
            quote! { alloc::borrow::Cow::Borrowed(#id) }
        } else {
            quote! { alloc::borrow::Cow::Owned(vec![#(#elts),*]) }
        }
    }

    fn attrs(&mut self, attrs: &[syn::Attribute]) -> Vec<TokenStream> {
        attrs
            .iter()
            .enumerate()
            .map(|(ix, a)| self.meta(ix, &a.parse_meta().expect("expected meta in attribute")))
            .collect()
    }

    fn field(&mut self, ix: usize, f: &'a syn::Field) -> TokenStream {
        let attrs_name = format_ident!(
            "{}_{}_ATTRS",
            self.parent(),
            f.ident
                .clone()
                .map_or_else(|| ix.to_string(), |f| f.to_string())
        );
        let ix = syn::Index::from(ix);

        let attrs = self.attrs(&f.attrs);
        self.consts.push(
            quote! { const #attrs_name : &'static [_reflect::Attr<'static>] = & [#(#attrs),*] },
        );
        let attrs = self.list_reference(attrs_name, attrs);

        let offset = match &f.ident {
            Some(name) => {
                quote! { memoffset::offset_of!(Self, #name) }
            }
            None => {
                quote! { memoffset::offset_of!(Self, #ix) }
            }
        };

        let field_ty = &f.ty;
        let shape = self.visit_ty(offset.clone(), field_ty);

        match &f.ident {
            Some(name) => quote! {
                _reflect::Field {
                    offset: #offset,
                    shape: #shape,
                    name: stringify!(#name),
                    attrs: #attrs,
                }
            },
            None => {
                quote! {
                    _reflect::TupleField {
                        offset: #offset,
                        shape: #shape,
                        attrs: #attrs,
                    }
                }
            }
        }
    }

    /// having traversed through all of the "item" components of a type declaration,
    /// we find ourselves at the very bottom looking at a field containing a structural rust type.
    fn visit_ty(&mut self, offset: TokenStream, ty: &'a syn::Type) -> TokenStream {
        match ty {
            syn::Type::Array(arr) => {
                let nested_shape = self.visit_ty(offset, &arr.elem);
                let len = &arr.len;
                quote! { _reflect::DataShape::FixedArray(#nested_shape, #len)}
            }
            syn::Type::Paren(syn::TypeParen { elem, .. })
            | syn::Type::Group(syn::TypeGroup { elem, .. }) => self.visit_ty(offset, elem),
            syn::Type::Infer(_) => panic!("are these even allowed where we're parsing for types?"),
            syn::Type::Path(path) => {
                let p = path.to_token_stream();
                quote! {
                    _reflect::_builtin_search(#path)
                                .unwrap_or(_reflect::DataShape::Leaf(core::any::TypeId::of::<#path>()))
                }
            }
            syn::Type::Reference(refer) => {
                let nested = self.visit_ty(quote! { 0 }, &refer.elem);
                quote! { _reflect::DataShape::Ref(alloc::borrow::Cow::Owned(#nested)) }
            }
            syn::Type::Slice(slice) => {
                let nested = self.visit_ty(quote! { 0 }, &slice.elem);
                quote! { _reflect::DataShape::Slice(alloc::borrow::Cow::Owned(#nested)) }
            }
            syn::Type::Tuple(tuple) => {
                let tup_elts = tuple
                    .elems
                    .iter()
                    .enumerate()
                    .map(|(ix, tp)| {
                        let ix = syn::Index::from(ix);
                        self.visit_ty(quote! { memoffset::offset_of_tuple!(#offset, #ix) }, tp)
                    })
                    .collect::<Vec<_>>();
                let tup_elts_name = format_ident!("{}_TUP_ELTS", self.parent());
                self.consts.push(
                        quote! { const #tup_elts_name : &'static [_reflect::DataShape<'static>] = &[#(#tup_elts),*]; },
                    );

                let tup_elts = self.list_reference(tup_elts_name, tup_elts);
                quote! { _reflect::DataShape::Tuple(#tup_elts) }
            }

            syn::Type::__Nonexhaustive => unreachable!(),
            syn::Type::BareFn(_) => panic!("don't reflect over function pointers"),
            m @ syn::Type::ImplTrait(_)
            | m @ syn::Type::TraitObject(_)
            | m @ syn::Type::Verbatim(_)
            | m @ syn::Type::Macro(_) => {
                self.seen_types.push(ty);
                quote! { _reflect::DataShape::Leaf(core::any::TypeId::of::<#m>() ) }
            }
            syn::Type::Ptr(_) => panic!("don't reflect over raw pointers"),
            syn::Type::Never(_) => panic!("don't reflect over never"),
        }
    }

    /*
        computing offsets for enum fields is busted.
        need to relayout the struct according to the repr/union system,
        and then get the layouts from that.
    */
    fn variant_data(&mut self, fields: &'a syn::Fields) -> TokenStream {
        let fields_ident = format_ident!("{}_FIELDS", self.parent());

        match fields {
            Fields::Named(n) => {
                let field_labels = n
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().expect("should be named").to_string());

                let fields = n
                    .named
                    .iter()
                    .enumerate()
                    .map(|(ix, f)| self.field(ix + 1, f))
                    .collect::<Vec<_>>();
                let field_labels_ident = format_ident!("{}_LABELS", fields_ident);

                self.consts.push(
                    quote! { const #fields_ident : &'static [&'static str] = &[#(#fields),*]; },
                );
                let fields = self.list_reference(fields_ident, fields);
                quote! { _reflect::VariantData::Fields {
                    labels_for_serde: { const #field_labels_ident : &'static [&'static str] = &[#(#field_labels),*]; #field_labels_ident },
                    fields: #fields
                } }
            }
            Fields::Unnamed(n) => {
                let fields = n
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(ix, f)| self.field(ix, f))
                    .collect::<Vec<_>>();
                self.consts.push(quote!{ const #fields_ident : &'static [_reflect::Field<'static>] = &[#(#fields),*]; } );

                let fields = self.list_reference(fields_ident, fields);
                quote! { _reflect::VariantData::Tuple(#fields) }
            }
            Fields::Unit => {
                quote! { _reflect::VariantData::Unit }
            }
        }
    }

    fn parented<T>(&mut self, new_parent: Ident, f: impl FnOnce(&mut Self) -> T) -> T {
        self.parent_stack.push(new_parent);
        let r = f(self);
        self.parent_stack.pop();
        r
    }

    /// For a variant `v` at index `ix` in its containing enum, generate the `EnumArm`
    /// reflecting it. `disc_base` is the index and defining expression of the last variant
    /// in this enum to have explicitly set a discriminant. It is returned, unless this variant
    /// itself has a discriminant set, in which case _that_ is returned.
    fn variant(&mut self, ix: usize, v: &'a syn::Variant) -> TokenStream {
        let attrs = self.parented(format_ident!("V{}", ix), |me| me.attrs(&v.attrs));

        let vdata = self.parented(format_ident!("{}", v.ident), |me| {
            me.variant_data(&v.fields)
        });
        let label = v.ident.to_string();
        let attrs_name = format_ident!("{}_{}_ATTRS", self.parent(), label);

        self.consts.push(
            quote! { const #attrs_name : &'static [_reflect::Attr<'static>] = &[#(#attrs),*]; },
        );

        let attrs = self.list_reference(attrs_name, attrs);

        let (new_disc_base, disc_val) = match &v.discriminant {
            Some((_eq_token, val)) => (Some((ix, val.to_token_stream())), val.to_token_stream()),
            None => (
                self.most_recent_discriminant_expr.clone(),
                match &self.most_recent_discriminant_expr {
                    Some((disc_setter_ix, prev_disc)) => {
                        quote! { #prev_disc + #(#ix - #disc_setter_ix) }
                    }
                    None => quote! { #ix },
                },
            ),
        };
        self.most_recent_discriminant_expr = new_disc_base;

        quote! {
            _reflect::EnumArm {
                label: #label,
                variant_index: #ix,
                discriminant: #disc_val,
                attrs: #attrs,
                variant: #vdata,
            }
        }
    }

    fn data_shape(&mut self, d: &'a syn::DeriveInput) -> TokenStream {
        match &d.data {
            Data::Struct(DataStruct { fields, .. }) => {
                let vdata = self.parented(d.ident.clone(), |me| me.variant_data(fields));
                quote! { _reflect::DataShape::Struct(#vdata) }
            }
            Data::Enum(DataEnum { variants, .. }) => {
                //let variant = variants.iter().map(|v| v.ident.to_string());
                let (labels, variants): (Vec<_>, Vec<_>) = variants
                    .iter()
                    .enumerate()
                    .map(|(ix, v)| {
                        let arm = self.parented(d.ident.clone(), |me| me.variant(ix, v));
                        (v.ident.to_string(), arm)
                    })
                    .unzip();
                let variants_ident = format_ident!("{}_VARIANTS", d.ident);
                let field_labels_ident = format_ident!("{}_LABELS", variants_ident);
                self.consts.push(
                    quote! { { const #field_labels_ident : &'static [&'static str] = &[#(#labels),*]; #field_labels_ident } },
                );
                self.consts.push(
                    quote! { const #variants_ident : &'static [_reflect::EnumArm<'static>] = &[#(#variants),*]},
                );
                let variants = self.list_reference(variants_ident, variants);

                quote! { _reflect::DataShape::Enum{
                    variant_labels_for_serde: #field_labels_ident,
                    variants: #variants,
                } }
            }
            Data::Union(_) => panic!("unions can not be reflected into"),
        }
    }
}

#[proc_macro_derive(Reflect)]
pub fn derive_reflect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse::<syn::DeriveInput>(input).unwrap();
    let mut derive = DeriveReflect {
        consts: vec![],
        nesting: None,
        parent_stack: vec![],
        most_recent_discriminant_expr: None,
        nightly_const: false,
        generics: &ast.generics,
        seen_types: vec![],
    };

    let attrs = derive.parented(ast.ident.clone(), |me| me.attrs(&ast.attrs));
    let shape = derive.data_shape(&ast);

    let leafs = vec![quote! {drop("oh well");}];

    // ideas for improving codegen:
    // - with a const offset_of, we can implement StaticReflect and not just SelfReflect
    // - how do we figure out when to register leafs?

    let attr_name = format_ident!("{}_ATTRS", ast.ident);
    derive
        .consts
        .push(quote! { const #attr_name : &'static [_reflect::Attr<'static>] = &[#(#attrs),*]; });

    // TODO: static_assert that every type either implements SelfReflect or implements Serialize/Deserialize

    let generics = &ast.generics;

    let me = &ast.ident;

    let consts = if derive.nightly_const {
        derive.consts
    } else {
        Vec::new()
    };

    let impl_block = quote! {
        const _ : () = {
            extern crate serde_reflect as _reflect;
            extern crate core;
            extern crate memoffset;
            extern crate alloc;

            #(#consts)*

            impl #generics _reflect::Reflect for #me #generics where Self: 'static {
                fn rust_type() -> _reflect::StaticType {
                    alloc::borrow::Cow::Owned(_reflect::ReflectedType {
                        id: core::any::TypeId::of::<Self>(),
                        name: stringify!(#me),
                        layout: core::alloc::Layout::new::<Self>(),
                        shape: #shape,
                        attrs: alloc::borrow::Cow::Owned(vec![#(#attrs),*]),
                    })
                }
                fn register(db: &mut _reflect::Db<'_,'_>) {
                    db.register_type::<Self>();
                    #(#leafs);*
                }
            }
        };
    };

    impl_block.into()
}
