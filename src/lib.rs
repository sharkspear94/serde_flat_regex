//!
//! A small macro for flattening map types with regex maching keys for struct fields and named enum fields.
//!
//! # Example
//!
//! ```
//! # use std::collections::HashMap;
//! use serde_flat_regex::flat_regex;
//! use serde::Deserialize;
//!
//! #[flat_regex]
//! #[derive(Debug,Deserialize)]   
//! struct RouterStatus {
//!     online: bool,
//!     #[flat_regex(regex = r"lanportstatus_\d+")]
//!     lanport_status: HashMap<String,bool>,
//! }
//!
//! let json = serde_json::json!({
//!     "online": true,
//!     "lanportstatus_0": true,
//!     "lanportspeed_0": "100", // no maching key will be ignored
//!     "lanportstatus_1": false,
//!     "lanportspeed_1": "0", // no maching key will be ignored
//!     "wifistatus": true
//! });
//! let res: RouterStatus = serde_json::from_value(json).unwrap();
//! assert_eq!(res.lanport_status.len(),2)
//! ```
//!
//! The collection for flattening must be a [serde-map type](https://docs.rs/serde/latest/serde/de/trait.MapAccess.html) and implement `Extend<(K,V)> + Default`.
//!
//! The key can be anything that implements `AsRef<str>` or alternitiv the field attribute `key_access` can be set with a function returning a `Result<&str,_>`.
//! The function has to have the following signature: `fn key_access_fn_name<T>(key: &T) -> Result<&str,Error>`.
//!
//! ```
//! # use std::collections::BTreeMap;
//! # use std::str::Utf8Error;
//! use std::ffi::CString;
//! use serde_flat_regex::flat_regex;
//! use serde::Deserialize;
//!
//! #[flat_regex]
//! #[derive(Debug,Deserialize)]   
//! struct RouterStatus {
//!     online: bool,
//!     #[flat_regex(regex = r"lanportstatus_\d+",key_access = "c_string_as_str")]
//!     lanport_status: BTreeMap<CString,bool>,
//! }
//!
//! fn c_string_as_str(key: &CString) -> Result<&str,Utf8Error> {
//!     key.to_str()
//! }
//!
//! let json = serde_json::json!({
//!     "online": true,
//!     "lanportstatus_0": true,
//!     "lanportspeed_0": "100", // no maching key will be ignored
//!     "lanportstatus_1": false,
//!     "lanport_speed_1": "0", // no maching key will be ignored
//!     "wifistatus": true
//! });
//! let res: RouterStatus = serde_json::from_value(json).unwrap();
//! assert_eq!(res.lanport_status.len(),2)
//! ```
//!

#![deny(missing_docs, unused_imports)]

extern crate proc_macro;
extern crate quote;
extern crate syn;

use darling::FromField;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Field, Fields, GenericArgument, Item,
    Lifetime, PathArguments, Type, TypeReference,
};

/// Macro for flattening map types with regex maching keys.
///
/// **Important**: The macro must be placed **before** deriving Deserialize
///
///
/// ```
/// # use std::collections::BTreeMap;
/// # use std::str::Utf8Error;
/// use std::ffi::CString;
/// use serde_flat_regex::flat_regex;
/// use serde::Deserialize;
///
/// #[flat_regex]
/// #[derive(Debug,Deserialize)]   
/// struct RouterStatus {
///     online: bool,
///     #[flat_regex(regex = r"lanportstatus_\d+")]
///     lanport_status: BTreeMap<String,bool>,
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn flat_regex(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    if let syn::Item::Struct(ref s) = item {
        let att = &s.attrs;
        let fields = &s.fields;
        let vis = &s.vis;
        let name = &s.ident;
        let gen = &s.generics;

        let mut with_fn = Vec::new();
        let fields = fields
            .iter()
            .map(|f| replace_attr(f, &name.to_string(), &mut with_fn));
        quote!(
                #(#att)*
                #vis struct #name #gen {
                    #(#fields,)*
                }

        #(#with_fn)*
        )
        .into()
    } else if let syn::Item::Enum(ref en) = item {
        let mut with_fn_vec = Vec::new();
        let name = &en.ident;
        let attrs = &en.attrs;
        let gen = &en.generics;
        let vis = &en.vis;

        let variants = en.variants.iter().map(|v| {
            let var_name = &v.ident;
            let att = &v.attrs;

            if let Fields::Named(a) = &v.fields {
                let fields = a
                    .named
                    .iter()
                    .map(|f| replace_attr(f, &format!("{name}_{var_name}"), &mut with_fn_vec));
                quote!(
                        #(#att)*
                        #var_name {
                            #(#fields),*
                        }
                )
            } else {
                quote!(#v)
            }
        });
        quote!(
            #(#attrs)*
            #vis enum #name #gen {
            #(#variants),*
            }

            #(#with_fn_vec)*
        )
        .into()
    } else {
        abort!(item, "only struct and enum supported")
    }
}

fn inner(ty: &syn::Type) -> (syn::GenericArgument, syn::GenericArgument) {
    let ret = match ty {
        syn::Type::Path(path) => {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                args: a, ..
            }) = &path.path.segments.last().unwrap().arguments
            {
                if a.len() == 2 {
                    (a[0].clone(), a[1].clone())
                } else {
                    abort!(
                        ty,
                        "type signature needs two Types, a key Type and value Type"
                    )
                }
            } else {
                abort!(ty, "type signature needs two Types, a key and value Type")
            }
        }
        _ => abort!(
            ty,
            "only angeled bracket types are supportet for flat_regex"
        ),
    };
    ret
}

fn lifetime(ty: &syn::GenericArgument) -> Option<Lifetime> {
    if let GenericArgument::Type(Type::Reference(TypeReference {
        lifetime: Some(lt), ..
    })) = ty
    {
        Some(lt.clone())
    } else {
        None
    }
}

#[derive(FromField)]
#[darling(attributes(flat_regex))]
struct FlatRegex {
    ident: Option<syn::Ident>,
    vis: syn::Visibility,
    ty: syn::Type,
    regex: String,
    key_access: Option<syn::ExprPath>,
}

fn replace_attr(
    field: &Field,
    prefix: &str,
    with_fn_vec: &mut Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let Ok(flat_field) = FlatRegex::from_field(field) else {
        return quote!(#field);
    };
    let ident = &flat_field.ident.unwrap();
    let vis = &flat_field.vis;
    let ty = &flat_field.ty;
    let attr = field
        .attrs
        .iter()
        .filter(|a| a.path.segments.last().unwrap().ident != "flat_regex")
        .map(|a| quote!(#a));

    let key_access = if let Some(fun) = &flat_field.key_access {
        quote!(let key_str = #fun(&key).map_err(A::Error::custom)?;)
    } else {
        quote!(let key_str = key.as_ref();)
    };

    let reg = &flat_field.regex;
    let s = {
        match regex::Regex::new(reg) {
            Ok(_) => (),
            Err(e) => {
                for f in &field.attrs {
                    if f.path.segments.last().unwrap().ident == "flat_regex" {
                        abort!(f.tokens, e.to_string())
                    }
                }
            }
        }

        let fun_name = format!("__with_regex_{prefix}_{ident}");
        let r = Ident::new(&format!("__with_regex_{prefix}_{ident}"), Span::call_site());

        // get inner generic values
        let (key, value) = inner(ty);
        let key_life = lifetime(&key);
        let val_life = lifetime(&value);

        let (with_lifetime, visitor_lifetime) = match (key_life, val_life) {
            (None, None) => (quote!('de), quote!()),
            (None, Some(v_lt)) => (quote!('de: #v_lt,#v_lt), quote!(#v_lt)),
            (Some(k_lt), None) => (quote!('de: #k_lt,#k_lt), quote!(#k_lt)),
            (Some(k_lt), Some(v_lt)) => {
                if k_lt == v_lt {
                    (quote!('de: #k_lt,#k_lt), quote!(#k_lt))
                } else {
                    (quote!('de: #v_lt+ #k_lt,#v_lt,#k_lt), quote!(#k_lt,#v_lt))
                }
            }
        };

        let collection = match ty {
            Type::Path(path) => path.path.segments.iter().map(|a| &a.ident),
            _ => abort!(ty, "somthing went wrong"),
        };

        with_fn_vec.push(quote!(
        fn #r<#with_lifetime, D,>(
            deserializer: D,
        ) -> std::result::Result<#ty, D::Error>
        where
            D: serde::Deserializer<'de>, {
            use serde::de::Error;

            struct RegexVisitor<#visitor_lifetime>(#ty);

            impl<#with_lifetime> serde::de::Visitor<'de> for RegexVisitor<#visitor_lifetime> {
                type Value = #ty;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "a {}",stringify!(#ty))
                }

                fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
                    where A: serde::de::MapAccess<'de>,
                {
                    let re = regex::Regex::new(#reg).unwrap();
                    while let Some(key) = map.next_key::<#key>()? {
                        #key_access
                        if re.is_match(key_str) {
                            let val = map.next_value::<#value>()?;
                            self.0.extend(std::iter::once((key, val)));
                        }
                    }
                    Ok(self.0)
                }
            }
        deserializer.deserialize_map(RegexVisitor((#(#collection::)*<#key,#value>::default())))
        }
        ));
        quote!(#[serde(flatten, deserialize_with = #fun_name)])
    };
    quote!(
        #s
        #(#attr)*
        #vis #ident: #ty
    )
}
