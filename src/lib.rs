//! # Serde Versions Derive
//! 
//!  `serde_versions_derive` exports an attribute macro that adds versioning support for structs.
//!  
//!  When serializing a named field struct it will automatically add a new field containing the version.
//!  It also allows deserializing the versioned type directly back to the unversioned one.
//!  
//!  Under the hood this works by creating a new struct that wraps the original struct plus adds a version byte field.
//!  Internally this new struct uses `#[serde(flatten)]` to serialize as expected.
//!  The original struct uses `#[serde(to, from)]` to add the version field when serializing and remove it when deserializing.
//!
//! usage: 
//! ```no_run
//! # use serde::{Deserialize, Serialize};
//! # use serde_versions_derive::version;
//! #[version(3)]
//! #[derive(Clone, Serialize, Deserialize)]
//! struct S {
//!     i: i32,
//! }
//! ```
//! 
//! This produces the following
//! ```ignore
//! #[derive(Clone, Serialize, Deserialize)]
//! #[serde(into = "_Sv3", from = "_Sv3")]
//! struct S {
//!     i: i32,
//! }
//! 
//! #[derive(Clone, Serialize, Deserialize)]
//! struct _Sv3 {
//!     version: u8,
//!     #[serde(flatten)]
//!     inner: S
//! }
//! 
//! // plus implementations of To, From and to_versioned() for S
//! ```
//! 
//! Note due to limitations of `#[serde(to, from)]` this does not support structs with type parameters.
//!  

use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::{parse::Parser, parse_macro_input, DeriveInput, LitInt};

/// Generate a new struct with a version field and ensure this struct is converted to that form before
/// serialization.
/// 
/// See crate doc for example.
/// 
#[proc_macro_attribute]
pub fn version(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_ast = parse_macro_input!(item as DeriveInput);
    let mut versioned_ast = original_ast.clone();

    let generics = original_ast.generics.clone();
    let version = parse_macro_input!(attr as LitInt);
    let struct_name = original_ast.ident.clone();

    // name is old struct name with V<version_number> appended
    let versioned_name = format_ident!("_{}v{}", original_ast.ident, version.to_string());
    let versioned_name_str = versioned_name.to_string();
    versioned_ast.ident = versioned_name.clone();

    match &mut versioned_ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                // drop all the fields and replace with an `inner` and a `version`
                syn::Fields::Named(fields) => {
                    fields.named.clear();
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { pub version: u8 })
                            .unwrap(),
                    );
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { #[serde(flatten)] pub inner: #struct_name #generics })
                            .unwrap(),
                    );
                }
                _ => (),
            }

            return quote! {
                #[serde(into = #versioned_name_str, from = #versioned_name_str)]
                #original_ast

                #versioned_ast

                impl #generics #struct_name #generics {
                    pub fn to_versioned(self) -> #versioned_name #generics {
                        #versioned_name {
                            version: #version,
                            inner: self,
                        }
                    }
                }

                impl #generics std::convert::Into<#versioned_name #generics> for #struct_name #generics {
                    fn into(self) -> #versioned_name #generics {
                        self.to_versioned()
                    }
                }

                impl #generics std::convert::From<#versioned_name #generics> for #struct_name #generics {
                    fn from(s: #versioned_name #generics) -> #struct_name #generics {
                        s.inner
                    }
                }
            }
            .into();
        }
        _ => panic!("`version` has to be used with structs "),
    }
}
