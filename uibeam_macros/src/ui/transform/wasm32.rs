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

            }

            NodeTokens::TextNode(node_pieces) => {

            }
        }
    }

    quote! {}
}