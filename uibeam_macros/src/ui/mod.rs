mod parse;
mod transform;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens { nodes } = syn::parse2(input)?;

    let doctype_insertion = nodes.first().map(|node| match node {
        /* starting with <html>..., without <!DOCTYPE html> */        
        parse::NodeTokens::EnclosingTag { tag, .. } if tag.to_string() == "html" => Some(quote! {
            unsafe {::uibeam::UI::new_unchecked(&["<!DOCTYPE html>"], [])},
        }),
        _ => None,
    }).flatten();

    let nodes = nodes.into_iter().map(|node| {
        let (literals, expressions) = transform::transform(node);
        quote! {
            unsafe {::uibeam::UI::new_unchecked(
                &[#(#literals),*],
                [#(#expressions),*]
            )}
        }
    });

    Ok(quote! {
        <::uibeam::UI>::concat([
            #doctype_insertion
            #(#nodes),*
        ])
    })
}
