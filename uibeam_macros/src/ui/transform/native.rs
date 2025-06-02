use super::super::parse::{UIDirectives, NodeTokens, ContentPieceTokens, InterpolationTokens, AttributeTokens, AttributeValueTokens, AttributeValueToken};
use super::{Component, prop_for_event};
use proc_macro2::{TokenStream, Span};
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, ExprLit, Lit, LitStr, Type};

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

pub(crate) struct EventHandlerAnnotation {
    handler_expression: Expr,
    event_type: Type,
}
impl ToTokens for EventHandlerAnnotation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { handler_expression, event_type } = self;
        tokens.extend(quote! {
            {
                fn __<Handler: Fn(#event_type)>(handler: Handler) {}
                #[allow(unused_braces)]
                __(#handler_expression);
            }
        });
    }
}

/// Derives `({HTML-escaped literal pieces}, {interpolating expressions})`
/// from the `NodeTokens`
pub(crate) fn transform(
    directives: &UIDirectives,
    tokens: NodeTokens,
) -> (
    Vec<Piece>,
    Vec<Interpolation>,
    Vec<EventHandlerAnnotation>,
) {
    let (mut pieces, mut interpolations, mut ehannotations) = (Vec::new(), Vec::new(), Vec::new());

    let mut piece = Piece::none();

    fn handle_node_tokens(
        directives: &UIDirectives,
        node: NodeTokens,
        current_piece: &mut Piece,
        pieces: &mut Vec<Piece>,
        interpolations: &mut Vec<Interpolation>,
        ehannotations: &mut Vec<EventHandlerAnnotation>,
    ) {
        let (child_pieces, child_interpolations, child_ehannotation) = transform(directives, node);

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

        ehannotations.extend(child_ehannotation);
    }

    fn handle_attributes(
        attributes: Vec<AttributeTokens>,
        current_piece: &mut Piece,
        pieces: &mut Vec<Piece>,
        interpolations: &mut Vec<Interpolation>,
        ehannotations: &mut Vec<EventHandlerAnnotation>,
    ) {
        for AttributeTokens { name, value } in attributes {
            if let Some(event) = name.to_string().strip_prefix("on") {
                if let Some((_prop, event_type)) = prop_for_event(&event.to_ascii_lowercase()) {
                    if let Some(value) = value.map(|t| t.value) {
                        ehannotations.push(EventHandlerAnnotation {
                            handler_expression: syn::parse2(value.into_token_stream()).unwrap(),
                            event_type,
                        });
                    }
                }
                continue;
            }

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
        directives: &UIDirectives,
        content: Vec<ContentPieceTokens>,
        current_piece: &mut Piece,
        pieces: &mut Vec<Piece>,
        interpolations: &mut Vec<Interpolation>,
        ehannotations: &mut Vec<EventHandlerAnnotation>,
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
                    directives,
                    node,
                    current_piece,
                    pieces,
                    interpolations,
                    ehannotations,
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
                let (value, is_literal) = match &a.value {
                    None => {
                        (quote! {true}, false)
                    }
                    Some(AttributeValueTokens { value, .. }) => match value {
                        AttributeValueToken::StringLiteral(lit) => {
                            (lit.into_token_stream(), true)
                        }
                        AttributeValueToken::IntegerLiteral(lit) => {
                            (LitStr::new(&lit.base10_digits(), lit.span()).into_token_stream(), true)
                        }
                        AttributeValueToken::Interpolation(InterpolationTokens { rust_expression, .. }) => {
                            (rust_expression.into_token_stream(), false)
                        }
                    }
                };
                if is_literal {
                    quote! {
                        #[allow(unused_braces)]
                        #name: (#value).into(),
                    }
                } else {
                    quote! {
                        #name: #value,
                    }
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
            let component_bound = directives.component_bound.clone()
                .unwrap_or(parse_quote! { ::uibeam::Beam });
            syn::parse2(quote! {
                <#name as #component_bound>::render(#name {
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
                    &mut interpolations,
                    &mut ehannotations,
                );
                piece.join(Piece::new(">"));
                handle_content_pieces(
                    directives,
                    content,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations,
                    &mut ehannotations,
                );
                piece.join(Piece::new(format!("</{tag}>")));
            }

            NodeTokens::SelfClosingTag { _open, tag, attributes, _slash, _end } => {
                piece.join(Piece::new(format!("<{tag}")));
                handle_attributes(
                    attributes,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations,
                    &mut ehannotations,
                );
                piece.join(Piece::new("/>"));
            }

            NodeTokens::TextNode(node_pieces) => {
                handle_content_pieces(
                    directives,
                    node_pieces,
                    &mut piece,
                    &mut pieces,
                    &mut interpolations,
                    &mut ehannotations,
                );
            }
        }

        piece.commit(&mut pieces);
    }

    (pieces, interpolations, ehannotations)
}
