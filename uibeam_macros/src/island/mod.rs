use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(
    input: TokenStream,
) -> syn::Result<TokenStream> {
    Ok(quote! {
        #[wasm_bindgen::prelude::wasm_bindgen]
        #input

        // require Serialize ?
    })
}
