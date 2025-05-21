use super::super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens, AttributeValueToken};
use super::{Piece, Interpolation, Component};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, Lit, LitStr, ExprLit};


/// Derives `({HTML-escaped literal pieces}, {interpolating expressions})`
/// from the `NodeTokens`
pub(crate) fn transform(
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

    fn handle_content_pieces(
        content: Vec<ContentPieceTokens>,
        current_piece: &mut Piece,
        pieces: &mut Vec<Piece>,
        interpolations: &mut Vec<Interpolation>,
    ) {
        for c in content {
            match c {
                ContentPieceTokens::StaticText(text) => {
                    current_piece.join(if text.token().to_string().starts_with("r#") {
                        Piece::new(text.value())
                    } else {
                        Piece::new(uibeam_html::escape(&text.value()))
                    });
                }
                ContentPieceTokens::Interpolation(InterpolationTokens { _unsafe, rust_expression, .. }) => {
                    let (is_unsafe, is_lit_str) = (
                        _unsafe.is_some(),
                        matches!(rust_expression, Expr::Lit(ExprLit { lit: Lit::Str(_), .. })),
                    );
                    if is_lit_str {// specialize for string literal
                        let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = rust_expression else {unreachable!()};
                        current_piece.join(if is_unsafe {
                            Piece::new(lit_str.value())
                        } else {
                            Piece::new(uibeam_html::escape(&lit_str.value()))
                        });
                    } else {
                        current_piece.is_none().then(|| *current_piece = Piece::new_empty());
                        current_piece.commit(pieces);
                        interpolations.push(if is_unsafe {
                            Interpolation::UnsafeRawChildren(rust_expression)
                        } else {
                            Interpolation::Children(rust_expression)
                        });
                        *current_piece = Piece::new_empty();
                    }
                }
                ContentPieceTokens::Node(node) => handle_node_tokens(
                    node,
                    current_piece,
                    pieces,
                    interpolations,
                )
            }
        }
    }

    if let Some(Component { name, attributes, content }) = tokens.as_beam() {
        piece.join(Piece::new_empty());
        piece.commit(&mut pieces);
        interpolations.push(Interpolation::Children({
            let attributes = attributes.iter().map(|a| {
                let name = a.name.as_ident().expect("Component attribute name must be a valid Rust identifier");
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
                ::uibeam::Component::render(#name {
                    #(#attributes)*
                    #children
                })
            }).unwrap()
        }));
        piece.join(Piece::new_empty());
        piece.commit(&mut pieces);

    } else {
        match tokens {
            NodeTokens::Doctype {
                _open,
                _bang,
                _doctype,
                _html,
                _end,
            } => (/*
                Skip transforming here and later insert it to the output
                (in `expand` in mod.rs).
                This enables an optimization at performance by directly
                concatinating `<!DOCTYPE html>` at the begenning of `<html...`
                literal piece.
            */),
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
                handle_content_pieces(
                    content,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations
                );
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
                handle_content_pieces(
                    node_pieces,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations
                );
            }
        }

        piece.commit(&mut pieces);
    }

    (pieces, interpolations)
}
