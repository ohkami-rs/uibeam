mod parse;
mod transform;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens { mut nodes } = syn::parse2(input)?;

    #[cfg(feature = "laser")]
    let wasm32_ui = {
        let wasm32_nodes = nodes
            .clone()
            .into_iter()
            .map(|node| {
                let vdom_tokens = transform::wasm32::transform(node)?;
                Ok(quote! {
                    ::uibeam::UI::new_unchecked(#vdom_tokens)
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        quote! {
            <::uibeam::UI>::from_iter([#(#wasm32_nodes),*])
        }
    };

    let native_ui = {
        if nodes
            .first()
            .is_some_and(|node| matches!(node, parse::NodeTokens::Doctype { .. }))
        {
            nodes.remove(0);
        }

        let mut should_insert_doctype = nodes.first().is_some_and(|node| match node {
            /* starting with <html>..., without <!DOCTYPE html> */
            parse::NodeTokens::EnclosingTag { tag, .. }
                if tag.to_string().eq_ignore_ascii_case("html") =>
            {
                true
            }
            _ => false,
        });

        let native_nodes = nodes
            .into_iter()
            .map(|node| {
                let (mut literals, expressions, ehannotations) =
                    transform::server::transform(node)?;
                if should_insert_doctype {
                    literals
                        .first_mut()
                        .unwrap()
                        .edit(|lit| *lit = format!("<!DOCTYPE html>{lit}"));
                    should_insert_doctype = false;
                }

                let ehannotations = (!ehannotations.is_empty()).then(|| {
                    quote! {
                        if false {
                            #(#ehannotations)*
                        }
                    }
                });

                Ok(quote! {{
                    #ehannotations
                    unsafe {::uibeam::UI::new_unchecked(
                        &[#(#literals),*],
                        [#(#expressions),*]
                    )}
                }})
            })
            .collect::<syn::Result<Vec<_>>>()?;

        quote! {
            <::uibeam::UI>::concat([#(#native_nodes),*])
        }
    };

    #[cfg(not(feature = "laser"))]
    return Ok(native_ui);

    #[cfg(feature = "laser")]
    return Ok(quote! {
        {
            #[cfg(target_arch = "wasm32")] {#wasm32_ui}
            #[cfg(not(target_arch = "wasm32"))] {#native_ui}
        }
    });
}
