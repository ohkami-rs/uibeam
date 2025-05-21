use super::super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens, AttributeValueToken};
use super::{Piece, Interpolation, Component};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Lit, LitStr, ExprLit};

/// Derives Rust codes that builds an `uibeam::laser::VDom` expression
/// corresponded to the `UI!` input
pub(crate) fn transform(
    tokens: NodeTokens,
) -> TokenStream {
    let mut t = TokenStream::new();
    encode(&mut t, tokens);
    return t;

    fn encode(t: &mut TokenStream, tokens: NodeTokens) {
        if let Some(Component { name, attributes, content }) = tokens.as_beam() {
            
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
                    
                }
                
                NodeTokens::SelfClosingTag {
                    _open,
                    tag,
                    attributes,
                    _slash,
                    _end,
                } => {
                    let tag = tag.to_string();

                    let attributes = {
                        let kvs = attributes.into_iter().map(|AttributeTokens { name, value }| {
                            let name = name.to_string();
                            let value = match value {
                                None => {
                                    quote! {
                                        ::uibeam::laser::wasm_bindgen::JsValue::from("")
                                    }
                                }
                                Some(AttributeValueTokens { _eq, value }) => {
                                    let value = match value {
                                        AttributeValueToken::IntegerLiteral(i) => i.into_token_stream(),
                                        AttributeValueToken::StringLiteral(s) => s.into_token_stream(),
                                        AttributeValueToken::Interpolation(InterpolationTokens {
                                            _unsafe, _brace, rust_expression
                                        }) => rust_expression.into_token_stream()
                                    };
                                    quote! {
                                        ::uibeam::laser::wasm_bindgen::JsValue::from(#value)
                                    }
                                }
                            };
                            quote! {
                                (#name, #value)
                            }
                        });
                        quote! {
                            vec![#(#kvs),*]
                        }
                    };

                    (quote! {
                        ::uibeam::laser::VDom::new(
                            ::uibeam::laser::ElementType::tag(#tag),
                            #attributes,
                            Vec::new()   
                        )
                    }).to_tokens(t);
                }
                
                NodeTokens::TextNode(node_pieces) => {
                    
                }
            }
        }
    }
}