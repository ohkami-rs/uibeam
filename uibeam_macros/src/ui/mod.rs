mod parse;
mod transform;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens {
        #[allow(unused_variables)]
        directives,
        #[allow(unused_mut)]
        mut nodes,
    } = syn::parse2(input)?;

    if crate::cfg_hydrate()? {
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

        Ok(quote! {
            <::uibeam::UI>::from_iter([#(#uis),*])
        })
    } else {
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
                let is_html_tag = node.children_of_enclosing_tag("html").is_some();

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

        Ok(quote! {
            <::uibeam::UI>::concat([#(#uis),*])
        })
    }
}
