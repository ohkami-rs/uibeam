#![cfg(feature = "laser")]

use super::super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens, AttributeValueToken};
use super::{Component, prop_for_event};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{LitStr, Expr};

fn as_event_handler(name: &str, expression: &Expr) -> Option<(LitStr, TokenStream)> {
    prop_for_event(&*name.strip_prefix("on")?.to_ascii_lowercase())
        .map(|(prop, event)| (
            LitStr::new(&prop.to_string(), prop.span()),
            quote! {
                ::uibeam::laser::wasm_bindgen::closure::Closure::<dyn Fn(#event)>::new(
                    #expression
                ).into_js_value()
            }
        ))
}

/// Derives Rust codes that builds an `uibeam::laser::VNode` expression
/// corresponded to the `UI!` input
pub(crate) fn transform(
    tokens: NodeTokens,
) -> TokenStream {
    let mut t = TokenStream::new();
    encode(&mut t, tokens);
    return t;

    fn encode(t: &mut TokenStream, tokens: NodeTokens) {
        fn into_props(attributes: Vec<AttributeTokens>) -> TokenStream {
            let kvs = attributes.into_iter().map(|AttributeTokens { name, value }| {
                let name = name.to_string();
                match value {
                    None => {
                        quote! {
                            (#name, ::uibeam::laser::wasm_bindgen::JsValue::TRUE)
                        }
                    }
                    Some(AttributeValueTokens { _eq, value }) => match value {
                        AttributeValueToken::IntegerLiteral(i) => {
                            quote! {
                                (#name, ::uibeam::laser::wasm_bindgen::JsValue::from(#i))
                            }
                        },
                        AttributeValueToken::StringLiteral(s) => {
                            let s = LitStr::new(&uibeam_html::escape(&s.value()), s.span());
                            quote! {
                                (#name, ::uibeam::laser::wasm_bindgen::JsValue::from(#s))
                            }
                        },
                        AttributeValueToken::Interpolation(InterpolationTokens {
                            _unsafe, _brace, rust_expression
                        }) => {
                            match as_event_handler(&name, &rust_expression) {
                                Some((prop, event_handler)) => {
                                    quote! {
                                        (#prop, #event_handler)
                                    }
                                }
                                None => {
                                    quote! {
                                        (#name, ::uibeam::laser::wasm_bindgen::JsValue::from(
                                            ::uibeam::AttributeValue::from(#rust_expression)
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
            });
            quote! {
                vec![#(#kvs),*]
            }
        }

        fn into_children(content: Vec<ContentPieceTokens>) -> TokenStream {
            let children = content.into_iter().map(|piece| {
                match piece {
                    ContentPieceTokens::StaticText(text) => {
                        let text = if text.token().to_string().starts_with("r#") {
                            text
                        } else {
                            LitStr::new(&uibeam_html::escape(&text.value()), text.span())
                        };
                        quote! {
                            ::uibeam::laser::VNode::text(#text)
                        }
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens {
                        _unsafe, _brace, rust_expression
                    }) => {
                        let is_escape = syn::LitBool::new(_unsafe.is_none(), Span::call_site());
                        quote! {
                            ::uibeam::IntoChildren::<_, #is_escape>::into_children(
                                #rust_expression
                            ).into_vdom()
                        }
                    }
                    ContentPieceTokens::Node(n) => {
                        transform(n)
                    }
                }
            });

            quote! {
                vec![#(#children),*]
            }
        }

        if let Some(Component { name, attributes, content }) = tokens.as_beam() {
            let props = into_props(attributes.to_vec());

            let children = into_children(content.map(<[_]>::to_vec).unwrap_or_else(Vec::new));

            (quote! {
                ::uibeam::laser::VNode::new(
                    ::uibeam::laser::NodeType::component::<#name>(),
                    #props,
                    #children   
                )
            }).to_tokens(t);
            
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

                    let props = into_props(attributes);

                    let children = into_children(content);

                    (quote! {
                        ::uibeam::laser::VNode::new(
                            ::uibeam::laser::NodeType::tag(#tag),
                            #props,
                            #children   
                        )
                    }).to_tokens(t);
                }
                
                NodeTokens::SelfClosingTag {
                    _open,
                    tag,
                    attributes,
                    _slash,
                    _end,
                } => {
                    let tag = tag.to_string();

                    let props = into_props(attributes);

                    (quote! {
                        ::uibeam::laser::VNode::new(
                            ::uibeam::laser::NodeType::tag(#tag),
                            #props,
                            const {Vec::new()}
                        )
                    }).to_tokens(t);
                }
                
                NodeTokens::TextNode(node_pieces) => {
                    into_children(node_pieces)
                        .to_tokens(t);
                }
            }
        }
    }
}