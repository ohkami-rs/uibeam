use super::parse::{Beam, NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, LitStr, Expr};

pub(super) struct Piece {
    text: String,
    span: Option<Span>,
}
impl ToTokens for Piece {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(_) = self.span {
            LitStr::new(
                &self.text,
                // not using `self.span` to avoid syntax highlighting pieces as
                // str literals, which would be less readable
                Span::call_site()
            ).to_tokens(tokens);
        }
    }
}
impl Piece {
    fn none() -> Self {
        Self {
            text: String::new(),
            span: None,
        }
    }
    fn is_none(&self) -> bool {
        self.text.is_empty() && self.span.is_none()
    }

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

    let mut piece = Piece::none();

    fn handle_node_tokens(
        node: NodeTokens,
        current_piece: &mut Piece,
        pieces: &mut Vec<Piece>,
        interpolations: &mut Vec<Interpolation>,
    ) {
        let (child_pieces, child_interpolations) = transform(node);
        
        let mut child_pieces = child_pieces.into_iter();
        
        if let Some(first_child_piece) = child_pieces.next() {
            current_piece.push(&first_child_piece.text, first_child_piece.span.unwrap());
        }
        for i in child_interpolations {
            current_piece.commit(pieces);
            interpolations.push(i);
            *current_piece = child_pieces.next().unwrap();
        }
        
        #[cfg(debug_assertions)] {
            assert!(child_pieces.next().is_none());
        }
    }

    if let Some(Beam { tag, attributes, content }) = tokens.as_beam() {
        piece.push("", Span::call_site());
        piece.commit(&mut pieces);
        interpolations.push(Interpolation::Children({
            let ident = tag;
            let attributes = attributes.iter().map(|a| {
                let name = a.name.as_ident().expect("Beam attribute name must be a valid Rust identifier");
                let value = match &a.value {
                    AttributeValueTokens::StringLiteral(lit) => {
                        lit.into_token_stream()
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        rust_expression.into_token_stream()
                    }
                };
                quote! {
                    #name: #value.into(),
                }
            });
            let children = content
                .and_then(|c| c.iter().map(|c| c.span()).reduce(|s1, s2| joined_span!(s1, s2)))
                .and_then(|s| s.source_text())
                .and_then(|t| syn::parse_str::<TokenStream>(&t).ok())
                .map(|t| quote! {
                    children: ::uibeam::UI! { #t },
                });
            syn::parse2(quote! {
                ::uibeam::Beam::render(#ident {
                    #(#attributes)*
                    #children
                })
            }).unwrap()
        }));
        piece.push("", Span::call_site());
        piece.commit(&mut pieces);

    } else {
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
                    piece.push(format!(" {name}="), joined_span!(name.span(), _eq.span()));
                    match value {
                        AttributeValueTokens::StringLiteral(lit) => {
                            piece.push(
                                &format!("\"{}\"", uibeam_html::html_escape(&lit.value())),
                                lit.span()
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
                        ContentPieceTokens::Node(node) => handle_node_tokens(
                            node,
                            &mut piece,
                            &mut pieces,
                            &mut interpolations,
                        ),
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
                    piece.push(format!(" {name}="), joined_span!(name.span(), _eq.span()));
                    match value {
                        AttributeValueTokens::StringLiteral(lit) => {
                            piece.push(
                                &format!("\"{}\"", uibeam_html::html_escape(&lit.value())),
                                lit.span()
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
                let mut last_was_interplolation = false;

                for np in node_pieces {
                    match np {
                        ContentPieceTokens::StaticText(text) => {
                            last_was_interplolation = false;
                            piece.push(
                                uibeam_html::html_escape(&text.value()),
                                text.span()
                            );
                        }
                        ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, _brace }) => {
                            if last_was_interplolation {
                                #[cfg(debug_assertions)] {// commited by the last interpolation
                                    assert!(piece.is_none());
                                }
                                Piece::commit(
                                    &mut Piece::new(String::new(), _brace.span.span()),
                                    &mut pieces
                                );
                            } else {
                                piece.commit(&mut pieces);
                            }
                            interpolations.push(Interpolation::Children(rust_expression));
                            last_was_interplolation = true;
                        }
                        ContentPieceTokens::Node(node) => handle_node_tokens(
                            node,
                            &mut piece,
                            &mut pieces,
                            &mut interpolations,
                        ),
                    }
                }
            }
        }

        piece.commit(&mut pieces);
    }

    (pieces, interpolations)
}
