mod parse;
mod transform;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens { mut nodes } = syn::parse2(input)?;

    if nodes.first().is_some_and(|node| matches!(node, parse::NodeTokens::Doctype { .. })) {
        nodes.remove(0);
    }

    let mut should_insert_doctype = nodes.first().is_some_and(|node| match node {
        /* starting with <html>..., without <!DOCTYPE html> */        
        parse::NodeTokens::EnclosingTag { tag, .. } if tag.to_string() == "html" => true,
        _ => false,
    });

    let nodes = nodes.into_iter().map(|node| {
        let (mut literals, expressions) = transform::transform(node);
        if should_insert_doctype {
            literals.first_mut().unwrap().edit(|lit| *lit = format!("<!DOCTYPE html>{lit}"));
            should_insert_doctype = false;
        }
        quote! {
            unsafe {::uibeam::UI::new_unchecked(
                &[#(#literals),*],
                [#(#expressions),*]
            )}
        }
    });

    Ok(quote! {
        <::uibeam::UI>::concat([#(#nodes),*])
    })
}
