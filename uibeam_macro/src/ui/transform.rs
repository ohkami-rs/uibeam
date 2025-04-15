use super::parse::{UITokens, NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Token, LitStr, Expr};
use syn::punctuated::Punctuated;

/// Derives `({literal prices}, {interpolated expressions})`
/// from the `NodeTokens`
pub(super) fn transform(
    tokens: NodeTokens,
) -> (
    Vec<LitStr>,
    Vec<Expr>,
) {
    todo!()
}
