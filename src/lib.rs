use proc_macro::TokenStream;
use quote::{format_ident, quote};

use syn::{parse::Parser, parse_macro_input, DeriveInput, LitInt};

#[proc_macro_attribute]
pub fn version(attr: TokenStream, item: TokenStream) -> TokenStream {
    let original_ast = parse_macro_input!(item as DeriveInput);
    let mut versioned_ast = original_ast.clone();

    let generics = original_ast.generics.clone();
    let version = parse_macro_input!(attr as LitInt);
    let struct_name = original_ast.ident.clone();

    // name is old struct name with V<version_number> appended
    let versioned_name = format_ident!("{}V{}", original_ast.ident, version.to_string());
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
                    // little bit of a hack. Original struct must derive Serialize and/or Deserialize
                    fields.named.push(
                        syn::Field::parse_named
                            .parse2(quote! { #[serde(flatten)] pub inner: #struct_name #generics })
                            .unwrap(),
                    );
                }
                _ => (),
            }

            return quote! {
                #[serde(into = "SV1", from = "SV1")]
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
        _ => panic!("`add_field` has to be used with structs "),
    }
}
