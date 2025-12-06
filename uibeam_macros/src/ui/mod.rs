mod parse;
mod transform;

use self::parse::{ContentPieceTokens, NodeTokens};

use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let parse::UITokens {
        directives,
        mut nodes,
    } = syn::parse2(input)?;

    #[cfg(hydrate)]
    {
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

        #[allow(clippy::needless_return)]
        return Ok(quote! {
            <::uibeam::UI>::from_iter([#(#uis),*])
        });
    }

    #[cfg(not(hydrate))]
    {
        fn is_enclosing_tag(node: &NodeTokens, tag_name: &str) -> bool {
            match node {
                /* starting with <html>..., without <!DOCTYPE html> */
                NodeTokens::EnclosingTag { tag, .. }
                    if tag.to_string().eq_ignore_ascii_case(tag_name) =>
                {
                    true
                }
                _ => false,
            }
        }

        if nodes
            .first()
            .is_some_and(|node| matches!(node, NodeTokens::Doctype { .. }))
        {
            // removing existing doctype declaration to insert our own later
            // as a part of static string literal (for performance optimization)
            nodes.remove(0);
        }

        let uis = nodes
            .into_iter()
            .map(|mut node| {
                if is_enclosing_tag(&node, "head") {
                    let NodeTokens::EnclosingTag { content, .. } = &mut node else {
                        unreachable!();
                    };
                    content.insert(
                        0,
                        ContentPieceTokens::Node(syn::parse_quote! {
                            <script type="importmap">
r#"{"imports": {
    "preact": "https://esm.sh/preact@10.28.0",
    "preact/": "https://esm.sh/preact@10.28.0/",
    "@preact/signals": "https://esm.sh/@preact/signals@2.5.1?external=preact"
}}"#
                            </script>
                        }),
                    );
                }

                if is_enclosing_tag(&node, "body") {
                    let NodeTokens::EnclosingTag { content, .. } = &mut node else {
                        unreachable!();
                    };
                    content.push(ContentPieceTokens::Node(syn::parse_quote! {
                        <script type="module" src="/.uibeam/hydrate.js"></script>
                    }));
                }

                let is_html_tag = is_enclosing_tag(&node, "html");

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

        #[allow(clippy::needless_return)]
        return Ok(quote! {
            <::uibeam::UI>::concat([#(#uis),*])
        });
    };
}
