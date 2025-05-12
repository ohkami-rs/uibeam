use super::parse::{NodeTokens, ContentPieceTokens, HtmlIdent, InterpolationTokens, AttributeTokens, AttributeValueTokens, AttributeValueToken};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{LitStr, Expr};

struct Beam<'n> {
    tag: &'n HtmlIdent,
    attributes: &'n [AttributeTokens],
    content: Option<&'n [ContentPieceTokens]>,
}
impl NodeTokens {
    fn as_beam(&self) -> Option<Beam<'_>> {
        let is_beam_ident = |html_ident: &HtmlIdent| {
            html_ident.as_ident().is_some_and(|ident| {
                ident.to_string().chars().next().unwrap().is_ascii_uppercase()
            })
        };
        match self {
            NodeTokens::EnclosingTag { tag, attributes, content, .. } => {
                is_beam_ident(tag).then_some(Beam {
                    tag,
                    attributes,
                    content: Some(content),
                })
            }
            NodeTokens::SelfClosingTag { tag, attributes, .. } => {
                is_beam_ident(tag).then_some(Beam {
                    tag,
                    attributes,
                    content: None,
                })
            }
            _ => None,
        }
    }
}

pub(super) struct Piece(Option<String>);
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
            current_piece.join(first_child_piece);
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

    fn handle_attributes(
        attributes: Vec<AttributeTokens>,
        current_piece: &mut Piece,
        pieces: &mut Vec<Piece>,
        interpolations: &mut Vec<Interpolation>,
    ) {
        for AttributeTokens { name, value } in attributes {
            current_piece.join(Piece::new(format!(" {name}")));
            if let Some(value) = value {
                current_piece.join(Piece::new("="));
                match value.value {
                    AttributeValueToken::StringLiteral(lit) => {
                        current_piece.join(Piece::new(format!(
                            "\"{}\"",
                            uibeam_html::escape(&lit.value())
                        )));
                    }
                    AttributeValueToken::IntegerLiteral(lit) => {
                        // escape is not needed for integer literals
                        current_piece.join(Piece::new(format!(
                            "\"{}\"",
                            lit.base10_digits()
                        )));
                    }
                    AttributeValueToken::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        current_piece.commit(pieces);
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }
        }
    }

    if let Some(Beam { tag, attributes, content }) = tokens.as_beam() {
        piece.join(Piece::new_empty());
        piece.commit(&mut pieces);
        interpolations.push(Interpolation::Children({
            let ident = tag;
            let attributes = attributes.iter().map(|a| {
                let name = a.name.as_ident().expect("Beam attribute name must be a valid Rust identifier");
                let value = match &a.value {
                    None => quote! {
                        true
                    },
                    Some(AttributeValueTokens { value, .. }) => match value {
                        AttributeValueToken::StringLiteral(lit) => {
                            lit.into_token_stream()
                        }
                        AttributeValueToken::IntegerLiteral(lit) => {
                            LitStr::new(&lit.base10_digits(), lit.span()).into_token_stream()
                        }
                        AttributeValueToken::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                            rust_expression.into_token_stream()
                        }
                    }
                };
                quote! {
                    #name: #value.into(),
                }
            });
            let children = content.map(|c| {
                let children_tokens = c.iter()
                    .map(ToTokens::to_token_stream)
                    .collect::<TokenStream>();
                quote! {
                    children: ::uibeam::UI! { #children_tokens },
                }
            });
            syn::parse2(quote! {
                ::uibeam::Beam::render(#ident {
                    #(#attributes)*
                    #children
                })
            }).unwrap()
        }));
        piece.join(Piece::new_empty());
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
                piece.join(Piece::new(format!("<{tag}")));
                handle_attributes(
                    attributes,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations
                );
                piece.join(Piece::new(">"));
                for c in content {
                    match c {
                        ContentPieceTokens::StaticText(text) => {
                            piece.join(Piece::new(
                                uibeam_html::escape(&text.value())
                            ));
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
                piece.join(Piece::new(format!("</{tag}>")));
            }

            NodeTokens::SelfClosingTag { _open, tag, attributes, _slash, _end } => {
                piece.join(Piece::new(format!("<{tag}")));
                handle_attributes(
                    attributes,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations
                );
                piece.join(Piece::new("/>"));
            }

            NodeTokens::TextNode(node_pieces) => {
                // let mut last_was_not_interplolation = pieces.is_empty();

                for np in node_pieces {
                    match np {
                        ContentPieceTokens::StaticText(text) => {
                            // last_was_not_interplolation = true;
                            piece.join(Piece::new(
                                uibeam_html::escape(&text.value())
                            ));
                        }
                        ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, _brace }) => {
                            // if last_was_not_interplolation {
                            //     #[cfg(debug_assertions)] {// commited by the last interpolation
                            //         assert!(piece.is_none());
                            //     }
                            //     Piece::new_empty().commit(&mut pieces);
                            // } else {
                            //     piece.commit(&mut pieces);
                            // }
                            let last_piece_is_none = piece.is_none();
                            if last_piece_is_none {
                                Piece::new_empty().commit(&mut pieces);
                            } else {
                                piece.commit(&mut pieces);
                            }
                            interpolations.push(Interpolation::Children(rust_expression));
                            piece = Piece::new_empty();
                            // last_was_not_interplolation = false;
                        }
                        ContentPieceTokens::Node(node) => {
                            // last_was_not_interplolation = true;
                            handle_node_tokens(
                                node,
                                &mut piece,
                                &mut pieces,
                                &mut interpolations,
                            );
                        }
                    }
                }
            }
        }

        piece.commit(&mut pieces);
    }

    (pieces, interpolations)
}
