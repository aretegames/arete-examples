//! READ ONLY.
//!
//! This crate provides the proc macros which allow you to derive the `Component` and `Resource` traits,
//! as well as the `#[system]` and `#[system_once]` attributes.

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, LitStr};

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let cid = Ident::new(
        &("_".to_string() + &ident.to_string().to_uppercase() + "_CID"),
        Span::call_site(),
    );

    let sid = LitStr::new(&ident.to_string(), Span::call_site());

    quote!(
        static mut #cid: ComponentId = 0;

        impl Component for #ident {
            fn id() -> ComponentId {
                unsafe { #cid }
            }

            fn set_id(id: ComponentId) {
                unsafe {
                    #cid = id;
                }
            }

            fn string_id() -> &'static std::ffi::CStr {
                unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(module_path!(), "::", #sid, "\0").as_bytes()) }
            }
        }

        impl Copy for #ident {}

        impl Clone for #ident {
            fn clone(&self) -> Self {
                *self
            }
        }
    )
    .into()
}

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let cid = Ident::new(
        &("_".to_string() + &ident.to_string().to_uppercase() + "_CID"),
        Span::call_site(),
    );

    let sid = LitStr::new(&ident.to_string(), Span::call_site());

    quote!(
        static mut #cid: ComponentId = 0;

        impl Resource for #ident {
            fn new() -> Self {
                Self::default()
            }

            fn id() -> ComponentId {
                unsafe { #cid }
            }

            fn set_id(id: ComponentId) {
                unsafe {
                    #cid = id;
                }
            }

            fn string_id() -> &'static std::ffi::CStr {
                unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(module_path!(), "::", #sid, "\0").as_bytes()) }
            }
        }
    )
    .into()
}

/// `system_once` is a marker attribute for FFI codegen.
#[proc_macro_attribute]
pub fn system_once(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// `system` is a marker attribute for FFI codegen.
#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
