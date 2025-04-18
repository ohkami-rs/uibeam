use super::parse::{UITokens, NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::LitStr;
use syn::spanned::Spanned;

pub(super) struct Piece(LitStr);
impl ToTokens for Piece {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Piece(lit_str) = self;
        let lit_str = lit_str.value();
        let lit_str = uibeam_html::html_escape(&lit_str);
        let lit_str = LitStr::new(&lit_str, lit_str.span());
        tokens.extend(quote! {
            #lit_str
        });
    }
}

pub(super) enum Interpolation {
    Attribute(TokenStream),
    Children(TokenStream),
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
    let (pieces, interpolations) = (Vec::new(), Vec::new());

    todo!();

    (pieces, interpolations)
}
