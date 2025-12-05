mod parse;
mod transform;

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens {
        directives,
        mut nodes,
    } = syn::parse2(input)?;

    #[cfg(hydrate)]
    return {
        let uis = nodes
            .clone()
            .into_iter()
            .map(|node| {
                let vdom_tokens = transform::hydrate::transform(&directives, node)?;
                Ok(quote! {
                    ::uibeam::UI::new_unchecked(#vdom_tokens)
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(quote! {
            <::uibeam::UI>::from_iter([#(#uis),*])
        })
    };

    #[cfg(not(hydrate))]
    return {
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

        let uis = nodes
            .into_iter()
            .map(|node| {
                let (mut literals, expressions, ehannotations) =
                    transform::server::transform(&directives, node)?;
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

        Ok(quote! {
            <::uibeam::UI>::concat([#(#uis),*])
        })
    };
}
