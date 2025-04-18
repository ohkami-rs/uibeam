use super::parse::{NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{LitStr, Expr};

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
    Vec<LitStr>,
    Vec<Interpolation>,
) {
    let (mut pieces, mut interpolations) = (Vec::new(), Vec::new());

    match tokens {
        NodeTokens::EnclosingTag { tag, attributes, content, .. } => {
            let mut piece = format!("<{tag}");

            macro_rules! commit_piece {
                () => {
                    pieces.push(LitStr::new(
                        &std::mem::take(&mut piece),
                        tag.span()
                    ));
                };
            }

            for AttributeTokens { name, value, .. } in attributes {
                match value {
                    AttributeValueTokens::StringLiteral(lit) => {
                        piece.push_str(&format!(
                            " {}=\"{}\"",
                            name,
                            uibeam_html::html_escape(&lit.value()),
                        ));
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        commit_piece!();
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }

            piece.push('>');

            for c in content {
                match c {
                    ContentPieceTokens::StaticText(text) => {
                        piece.push_str(&uibeam_html::html_escape(&text.value()));
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        commit_piece!();
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        let (child_pieces, child_interpolations) = transform(node);
                        pieces.extend(child_pieces);
                        interpolations.extend(child_interpolations);
                    }
                }
            }

            piece.push_str(&format!("</{tag}>"));

            commit_piece!();
        }

        NodeTokens::SelfClosingTag { tag, attributes, .. } => {
            let mut piece = format!("<{tag}");

            macro_rules! commit_piece {
                () => {
                    pieces.push(LitStr::new(
                        &std::mem::take(&mut piece),
                        tag.span()
                    ));
                };
            }

            for AttributeTokens { name, value, .. } in attributes {
                match value {
                    AttributeValueTokens::StringLiteral(lit) => {
                        piece.push_str(&format!(
                            " {}=\"{}\"",
                            name,
                            uibeam_html::html_escape(&lit.value()),
                        ));
                    }
                    AttributeValueTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        commit_piece!();
                        interpolations.push(Interpolation::Attribute(rust_expression));
                    }
                }
            }

            piece.push_str("/>");

            commit_piece!();
        }

        NodeTokens::TextNode(node_pieces) => {
            let mut piece = String::new();
            macro_rules! commit_piece {
                () => {
                    pieces.push(LitStr::new(
                        &std::mem::take(&mut piece),
                        Span::call_site()
                    ));
                };
            }
            for np in node_pieces {
                match np {
                    ContentPieceTokens::StaticText(text) => {
                        piece.push_str(&uibeam_html::html_escape(&text.value()));
                    }
                    ContentPieceTokens::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                        commit_piece!();
                        interpolations.push(Interpolation::Children(rust_expression));
                    }
                    ContentPieceTokens::Node(node) => {
                        commit_piece!();
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
