#![cfg(feature = "client")]

use super::super::parse::{
    AttributeTokens, AttributeValueToken, AttributeValueTokens, ContentPieceTokens,
    InterpolationTokens, NodeTokens,
};
use super::prop_for_event;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Expr, LitStr};

fn as_event_handler(name: &str, expression: &Expr) -> Option<syn::Result<(LitStr, TokenStream)>> {
    name.strip_prefix("on").map(|event| {
        prop_for_event(&event.to_ascii_lowercase()).map(|(prop, event)| {
            (
                LitStr::new(&prop.to_string(), prop.span()),
                quote! {
                    ::uibeam::client::wasm_bindgen::closure::Closure::<dyn Fn(#event)>::new(
                        #expression
                    ).into_js_value()
                },
            )
        })
    })
}

/// Derives Rust codes that builds an `uibeam::client::VNode` expression
/// corresponded to the `UI!` input
pub(crate) fn codegen(tokens: NodeTokens) -> syn::Result<TokenStream> {
}
