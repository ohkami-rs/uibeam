use super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{LitStr, Expr};
use syn::spanned::Spanned;

pub(super) struct Piece(LitStr);
impl ToTokens for Piece {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Piece(lit_str) = self;
        let lit_str = lit_str.value();
        let lit_str = uibeam_html::html_escape(&lit_str);
        let lit_str = LitStr::new(&lit_str, lit_str.span());
        lit_str.to_tokens(tokens);
    }
}

pub(super) enum Interpolation {
    Attribute(Expr),
    Children(Expr),
}
impl ToTokens for Interpolation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Interpolation::Attribute(expression) => tokens.extend(quote! {
                ::uibeam::Interpolator::Attribute(::uibeam::AttributeValue::from(
                    #expression
                ))
            }),
            Interpolation::Children(expression) => tokens.extend(quote! {
                ::uibeam::Interpolator::Children(::uibeam::IntoChildren::into_children(
                    #expression
                ))
            }),
        }
    }
}

/// Derives `({HTML-escaped literal pieces}, {interpolating expressions})`
/// from the `NodeTokens`
pub(super) fn transform(
    tokens: NodeTokens,
) -> (
    Vec<Piece>,
    Vec<Interpolation>,
) {
    let (mut pieces, mut interpolations) = (Vec::new(), Vec::new());

    match tokens {
        NodeTokens::EnclosingTag { tag, attributes, content, .. } => {
            pieces.push(Piece(LitStr::new(&format!("<{tag}"), tag.span())));
            for AttributeTokens { name, value, .. } in attributes {
                match value {
                    AttributeValueTokens::StringLiteral(lit_str) => {
                        pieces.push(Piece(LitStr::new(&format!(" {}=\"{}\"", name, lit_str.value()), name.span())));
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }
            pieces.push(Piece(LitStr::new(">", tag.span())));
            for cp in content {
                match cp {
                    ContentPieceTokens::StaticText(lit_str) => {
                        pieces.push(Piece(lit_str));
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        let (child_pieces, child_interpolations) = transform(node);
                        pieces.extend(child_pieces);
                        interpolations.extend(child_interpolations);
                    }
                }
            }
            pieces.push(Piece(LitStr::new(&format!("</{tag}>"), tag.span())));
        }

        NodeTokens::SelfClosingTag { tag, attributes, .. } => {
            pieces.push(Piece(LitStr::new(&format!("<{tag}"), tag.span())));
            for AttributeTokens { name, value, .. } in attributes {
                match value {
                    AttributeValueTokens::StringLiteral(lit_str) => {
                        pieces.push(Piece(LitStr::new(&format!(" {}=\"{}\"", name, lit_str.value()), name.span())));
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }
            pieces.push(Piece(LitStr::new(" />", tag.span())));
        }

        NodeTokens::TextNode(node_pieces) => {
            for np in node_pieces {
                match np {
                    ContentPieceTokens::StaticText(lit_str) => {
                        pieces.push(Piece(lit_str));
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        let (child_pieces, child_interpolations) = transform(node);
                        pieces.extend(child_pieces);
                        interpolations.extend(child_interpolations);
                    }
                }
            }
        }
    }

    (pieces, interpolations)
}
