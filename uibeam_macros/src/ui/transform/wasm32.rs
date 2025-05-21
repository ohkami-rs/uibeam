use super::super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens, AttributeValueToken};
use super::{Piece, Interpolation, Component};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Lit, LitStr, ExprLit};

pub(crate) fn transform(
    tokens: NodeTokens,
) -> () {
    
}