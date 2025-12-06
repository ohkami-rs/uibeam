mod parse;
mod transform;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens {
        #[cfg_attr(hydrate, allow(unused_variables))]
        directives,
        #[cfg_attr(hydrate, allow(unused_mut))]
        mut nodes,
    } = syn::parse2(input)?;

    let hydrate_ui = {
        let uis = nodes
            .clone()
            .into_iter()
            .map(|node| {
                let vdom_tokens = transform::hydrate::transform(node)?;
                Ok(quote! {
                    ::uibeam::UI::new_unchecked(#vdom_tokens)
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        quote! {
            <::uibeam::UI>::from_iter([#(#uis),*])
        }
    };

    let server_ui = {
        if nodes
            .first()
            .is_some_and(|node| matches!(node, self::parse::NodeTokens::Doctype { .. }))
        {
            // removing existing doctype declaration to insert our own later
            // as a part of static string literal (for performance optimization)
            nodes.remove(0);
        }

        let uis = nodes
            .into_iter()
            .map(|node| {
                let is_html_tag = node.enclosing_tag_children("html").is_some();

                let (mut literals, expressions, ehannotations) =
                    transform::server::transform(&directives, node)?;

                if is_html_tag {
                    literals
                        .first_mut()
                        .unwrap()
                        .edit(|lit| *lit = format!("<!DOCTYPE html>{lit}"));
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
            <::uibeam::UI>::concat([#(#uis),*])
        }
    };

    Ok(quote! {{
        #[cfg(hydrate)]
        {
            #hydrate_ui
        }
        #[cfg(not(hydrate))]
        {
            #server_ui
        }
    }})
}
