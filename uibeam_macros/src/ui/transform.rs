pub(super) mod native;
pub(super) mod wasm32;

use super::parse::{NodeTokens, ContentPieceTokens, HtmlIdent, AttributeTokens};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{Expr, Ident, LitStr};

struct Component<'n> {
    name: &'n Ident,
    attributes: &'n [AttributeTokens],
    content: Option<&'n [ContentPieceTokens]>,
}
impl NodeTokens {
    fn as_beam(&self) -> Option<Component<'_>> {
        fn as_component_name(html_ident: &HtmlIdent) -> Option<&Ident> {
            html_ident
                .as_ident()
                .map(|ident| ident.to_string().chars().next().unwrap().is_ascii_uppercase().then_some(ident))
                .flatten()
        }
        match self {
            NodeTokens::EnclosingTag { tag, attributes, content, .. } => {
                as_component_name(tag).map(|name| Component {
                    name,
                    attributes,
                    content: Some(content),
                })
            }
            NodeTokens::SelfClosingTag { tag, attributes, .. } => {
                as_component_name(tag).map(|name| Component {
                    name,
                    attributes,
                    content: None,
                })
            }
            _ => None,
        }
    }
}

pub(crate) struct Piece(Option<String>);
impl ToTokens for Piece {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(text) = &self.0 {
            LitStr::new(text, Span::call_site()).to_tokens(tokens);
        }
    }
}
impl Piece {
    fn none() -> Self {
        Self(None)
    }
    fn is_none(&self) -> bool {
        self.0.is_none()
    }

    fn new_empty() -> Self {
        Self(Some(String::new()))
    }
    fn new(text: impl Into<String>) -> Self {
        Self(Some(text.into()))
    }

    fn join(&mut self, other: Piece) {
        match &mut self.0 {
            Some(text) => if let Some(other_text) = other.0 {
                text.push_str(&other_text);
            }
            None => {
                self.0 = other.0;
            }
        }
    }

    fn commit(&mut self, pieces: &mut Vec<Self>) {
        if let Some(text) = self.0.take() {
            pieces.push(Piece::new(text));
        }
    }

    pub(crate) fn edit(&mut self, f: impl FnOnce(&mut String)) {
        self.0.as_mut().map(f);
    }
}

pub(crate) enum Interpolation {
    Attribute(Expr),
    Children(Expr),
    UnsafeRawChildren(Expr),
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
                ::uibeam::Interpolator::Children(::uibeam::IntoChildren::<_, true>::into_children(
                    #expression
                ))
            }),
            Interpolation::UnsafeRawChildren(expression) => tokens.extend(quote! {
                ::uibeam::Interpolator::Children(::uibeam::IntoChildren::<_, false>::into_children(
                    #expression
                ))
            }),
        }
    }
}
