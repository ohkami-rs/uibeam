// just allow unused when not(hydrate), instead of applying `#![cfg(hydrate)]`, for DX.
#![cfg_attr(not(hydrate), allow(unused))]

use super::super::parse::{
    AttributeTokens, AttributeValueToken, AttributeValueTokens, ContentPieceTokens, Directive,
    InterpolationTokens, NodeTokens,
};
use super::{Component, prop_for_event};
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
pub(crate) fn transform(tokens: NodeTokens) -> syn::Result<TokenStream> {
    let mut t = TokenStream::new();
    encode(&mut t, tokens)?;
    return Ok(t);

    fn encode(t: &mut TokenStream, tokens: NodeTokens) -> syn::Result<()> {
        fn into_props(attributes: Vec<AttributeTokens>) -> syn::Result<TokenStream> {
            if attributes.is_empty() {
                return Ok(quote! {
                    ::uibeam::client::js_sys::Object::new()
                });
            }

            let kvs = attributes
                .into_iter()
                .map(|AttributeTokens { name, value }| {
                    let name = name.to_string();
                    match value {
                        None => Ok(quote! {
                            (#name, ::uibeam::client::wasm_bindgen::JsValue::TRUE)
                        }),
                        Some(AttributeValueTokens { _eq, value }) => match value {
                            AttributeValueToken::IntegerLiteral(i) => Ok(quote! {
                                (#name, ::uibeam::client::wasm_bindgen::JsValue::from(#i))
                            }),
                            AttributeValueToken::StringLiteral(s) => {
                                let s = LitStr::new(&uibeam_html::escape(&s.value()), s.span());
                                Ok(quote! {
                                    (#name, ::uibeam::client::wasm_bindgen::JsValue::from(#s))
                                })
                            }
                            AttributeValueToken::Interpolation(InterpolationTokens {
                                _unsafe,
                                _brace,
                                rust_expression,
                            }) => match as_event_handler(&name, &rust_expression) {
                                Some(eh) => {
                                    let (prop, event_handler) = eh?;
                                    Ok(quote! {
                                        (#prop, #event_handler)
                                    })
                                }
                                None => Ok(quote! {
                                    (#name, ::uibeam::client::wasm_bindgen::JsValue::from(
                                        ::uibeam::AttributeValue::from(#rust_expression)
                                    ))
                                }),
                            },
                        },
                    }
                })
                .collect::<syn::Result<Vec<_>>>()?;

            Ok(quote! {
                {
                    let props = ::uibeam::client::js_sys::Object::new();
                    for (k, v) in [#(#kvs),*] {
                        ::uibeam::client::js_sys::Reflect::set(&props, &k.into(), &v).unwrap();
                    }
                    props
                }
            })
        }

        fn into_children(content: Vec<ContentPieceTokens>) -> syn::Result<TokenStream> {
            let children = content
                .into_iter()
                .map(|piece| match piece {
                    ContentPieceTokens::StaticText(text) => {
                        let text = if text.token().to_string().starts_with("r#") {
                            text
                        } else {
                            LitStr::new(&uibeam_html::escape(&text.value()), text.span())
                        };
                        Ok(quote! {
                            ::uibeam::client::VNode::text(#text)
                        })
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens {
                        _unsafe,
                        _brace,
                        rust_expression,
                    }) => {
                        let is_escape = syn::LitBool::new(_unsafe.is_none(), Span::call_site());
                        Ok(quote! {
                            ::uibeam::IntoChildren::<_, #is_escape>::into_children(
                                #rust_expression
                            ).into_vdom()
                        })
                    }
                    ContentPieceTokens::Node(n) => transform(n),
                })
                .collect::<syn::Result<Vec<_>>>()?;

            Ok(quote! {
                vec![#(#children),*]
            })
        }

        if let Some(Component {
            name,
            attributes,
            content,
        }) = tokens.as_beam()
        {
            let props = into_props(attributes.to_vec())?;

            let children = into_children(content.map(<[_]>::to_vec).unwrap_or_else(Vec::new))?;

            (quote! {
                ::uibeam::client::VNode::new(
                    ::uibeam::client::NodeType::component::<#name>(),
                    #props,
                    #children
                )
            })
            .to_tokens(t);
        } else {
            match tokens {
                NodeTokens::Doctype { .. } => (/* ignore */),

                NodeTokens::EnclosingTag {
                    _start_open,
                    tag,
                    attributes,
                    _start_close,
                    content,
                    _end_open,
                    _slash,
                    _tag,
                    _end_close,
                } => {
                    let tag = tag.to_string();

                    let props = into_props(attributes)?;

                    let children = into_children(content)?;

                    (quote! {
                        ::uibeam::client::VNode::new(
                            ::uibeam::client::NodeType::tag(#tag),
                            #props,
                            #children
                        )
                    })
                    .to_tokens(t);
                }

                NodeTokens::SelfClosingTag {
                    _open,
                    tag,
                    attributes,
                    _slash,
                    _end,
                } => {
                    let tag = tag.to_string();

                    let props = into_props(attributes)?;

                    (quote! {
                        ::uibeam::client::VNode::new(
                            ::uibeam::client::NodeType::tag(#tag),
                            #props,
                            const {Vec::new()}
                        )
                    })
                    .to_tokens(t);
                }

                NodeTokens::TextNode(node_pieces) => {
                    into_children(node_pieces)?.to_tokens(t);
                }
            }
        }
        Ok(())
    }
}
