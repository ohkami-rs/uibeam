use super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, LitStr, Expr};

macro_rules! joined_span {
    ($span:expr $( , $other_span:expr )*) => {
        {
            let mut span: proc_macro2::Span = $span;
            $(
                span = span.join($other_span).unwrap_or(span);
            )+
            span
        }
    };
}

pub(super) struct Piece {
    text: String,
    span: Option<Span>,
}
impl ToTokens for Piece {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(span) = self.span {
            LitStr::new(&self.text, span).to_tokens(tokens);
        }
    }
}
impl Default for Piece {
    fn default() -> Self {
        Self {
            text: String::new(),
            span: None,
        }
    }
}
impl Piece {
    fn new(text: String, span: Span) -> Self {
        Self { text, span: Some(span) }
    }

    fn push(&mut self, text: impl AsRef<str>, span: Span) {
        match &mut self.span {
            Some(existing_span) => {
                *existing_span = existing_span.join(span).unwrap_or(*existing_span);
                self.text.push_str(text.as_ref());
            }
            None => {
                #[cfg(debug_assertions)] {
                    assert!(self.text.is_empty());
                }
                self.span = Some(span);
                self.text = text.as_ref().into();
            }
        }
    }

    fn commit(&mut self, pieces: &mut Vec<Self>) {
        if let Some(span) = self.span {
            self.span = None;
            pieces.push(Piece::new(std::mem::take(&mut self.text), span));
        } else {
            #[cfg(debug_assertions)] {
                assert!(self.text.is_empty());
            }
        }
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

    let mut piece = Piece::default();

    match tokens {
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
            piece.push(
                format!("<{tag}"),
                joined_span!(_start_open.span(), tag.span())
            );
            for AttributeTokens { name, _eq, value } in attributes {
                match value {
                    AttributeValueTokens::StringLiteral(lit) => {
                        piece.push(
                            &format!(" {}=\"{}\"", name, uibeam_html::html_escape(&lit.value())),
                            joined_span!(lit.span(), _eq.span, name.span())
                        );
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        piece.commit(&mut pieces);
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }
            piece.push(">", _start_close.span());
            for c in content {
                match c {
                    ContentPieceTokens::StaticText(text) => {
                        piece.push(&uibeam_html::html_escape(&text.value()), text.span());
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        piece.commit(&mut pieces);
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        let (child_pieces, child_interpolations) = transform(node);
                        let mut child_pieces = child_pieces.into_iter();
                        if let Some(first_child_piece) = child_pieces.next() {
                            piece.push(&first_child_piece.text, first_child_piece.span.unwrap());
                        }
                        pieces.extend(child_pieces);
                        interpolations.extend(child_interpolations);
                    }
                }
            }
            piece.push(
                &format!("</{tag}>"),
                joined_span!(_end_open.span(), _slash.span(), _tag.span(), _end_close.span())
            );
        }

        NodeTokens::SelfClosingTag { _open, tag, attributes, _slash, _end } => {
            piece.push(format!("<{tag}"), joined_span!(_open.span(), tag.span()));
            for AttributeTokens { name, _eq, value } in attributes {
                match value {
                    AttributeValueTokens::StringLiteral(lit) => {
                        piece.push(
                            &format!(" {}=\"{}\"", name, uibeam_html::html_escape(&lit.value())),
                            joined_span!(lit.span(), _eq.span, name.span())
                        );
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        piece.commit(&mut pieces);
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }
            piece.push("/>", joined_span!(_slash.span(), _end.span()));
        }

        NodeTokens::TextNode(node_pieces) => {
            for np in node_pieces {
                match np {
                    ContentPieceTokens::StaticText(text) => {
                        piece.push(
                            uibeam_html::html_escape(&text.value()),
                            text.span()
                        );
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        piece.commit(&mut pieces);
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        let (child_pieces, child_interpolations) = transform(node);
                        let mut child_pieces = child_pieces.into_iter();
                        if let Some(first_child_piece) = child_pieces.next() {
                            piece.push(&first_child_piece.text, first_child_piece.span.unwrap());
                        }
                        pieces.extend(child_pieces);
                        interpolations.extend(child_interpolations);
                    }
                }
            }
        }
    }

    piece.commit(&mut pieces);

    (pieces, interpolations)
}
