use super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, LitStr, Expr};

macro_rules! joined_span {
    ($span:expr $( , $other_span:expr )*) => {
        {
            let mut span = $span;
            $(
                span = span.join($other_span).unwrap_or(span);
            )+
            span
        }
    };
}

pub(super) struct Piece {
    text: String,
    span: Span,
}
impl ToTokens for Piece {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        LitStr::new(&self.text, self.span).to_tokens(tokens);
    }
}
impl Piece {
    fn new(text: String, span: Span) -> Self {
        Self { text, span }
    }

    fn push(&mut self, text: &str, span: Span) {
        self.text.push_str(text);
        self.span = joined_span!(self.span, span);
    }

    fn commit(&mut self, pieces: &mut Vec<Self>) {
        pieces.push(Piece::new(std::mem::take(&mut self.text), self.span));
        self.span = Span::call_site();
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
            let mut piece = Piece::new(
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
                        pieces.extend(child_pieces);
                        interpolations.extend(child_interpolations);
                    }
                }
            }
            piece.push(
                &format!("</{tag}>"),
                joined_span!(_end_open.span(), _slash.span(), _tag.span(), _end_close.span())
            );
            piece.commit(&mut pieces);
        }

        NodeTokens::SelfClosingTag { _open, tag, attributes, _slash, _end } => {
            let mut piece = Piece::new(
                format!("<{tag}"),
                joined_span!(_open.span(), tag.span())
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
            piece.push("/>", joined_span!(_slash.span(), _end.span()));
            piece.commit(&mut pieces);
        }

        NodeTokens::TextNode(node_pieces) => {
            let mut piece = None::<Piece>;
            for np in node_pieces {
                match np {
                    ContentPieceTokens::StaticText(text) => {
                        let (text, span) = (text.value(), text.span());
                        let text = uibeam_html::html_escape(&text);
                        match piece.as_mut() {
                            Some(piece) => piece.push(&text, span),
                            None => piece = Some(Piece::new(text.into_owned(), span)),
                        }
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        if let Some(mut piece) = piece.take() {
                            piece.commit(&mut pieces);
                        }
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        if let Some(mut piece) = piece.take() {
                            piece.commit(&mut pieces);
                        }
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
