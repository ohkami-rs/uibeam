#![cfg(feature = "laser")]

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(super) fn expand(
    _args: TokenStream,
    input: TokenStream,
) -> syn::Result<TokenStream> {
    let input: syn::ItemStruct = syn::parse2(input)?;

    let name = &input.ident;
    let hydrater_name = format_ident!("__uibeam_laser_{name}__");
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        #[::uibeam::laser::wasm_bindgen::prelude::wasm_bindgen]
        #input

        #[cfg(target_arch = "wasm32")]
        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[::uibeam::laser::wasm_bindgen::prelude::wasm_bindgen]
        pub fn #hydrater_name #impl_generics(props: #name #ty_generics, container: ::uibeam::laser::web_sys::Node)
            #where_clause
        {
            ::uibeam::laser::hydrate(
                <#name as ::uibeam::Laser>::render(props).into_vdom(),
                container
            )
        }
    })
}
