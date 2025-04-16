use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{token, Token, Ident, Expr, LitStr};

/// Parsed representation of the UI macro input.
/// 
/// This is almost HTML syntax, but with some Rust expressions embedded within `{}`.
pub(super) struct UITokens {
    pub(super) nodes: Vec<NodeTokens>,
}

pub(super) enum NodeTokens {
    EnclosingTag {
        _start_open: Token![<],
        tag: Ident,
        attributes: Vec<AttributeTokens>,
        _start_close: Token![>],
        content: ContentPiecesTokens,
        _end_open: Token![<],
        _slash: Token![/],
        _tag: Ident,
        _end_close: Token![>],
    },
    SelfClosingTag {
        _open: Token![<],
        tag: Ident,
        attributes: Vec<AttributeTokens>,
        _slash: Token![/],
        _end: Token![>],
    },
    TextNode {
        pieces: ContentPiecesTokens,
    },
}

pub(super) struct ContentPiecesTokens(
    Vec<ContentPiece>
);
enum ContentPiece {
    Interpolation(InterpolationTokens),
    Node(NodeTokens),
    Content(TokenStream),
}

pub(super)struct InterpolationTokens {
    _brace: token::Brace,
    pub(super) rust_expression: Expr,
}

pub(super)struct AttributeTokens {
    pub(super) name: Ident,
    _eq: Token![=],
    pub(super) value: AttributeValueTokens,
}

pub(super) enum AttributeValueTokens {
    StringLiteral(LitStr),
    Interpolation(InterpolationTokens),
}

impl Parse for UITokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }
        Ok(UITokens { nodes })
    }
}

impl Parse for NodeTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

impl Parse for ContentPiecesTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace) {
            return Ok(Self(vec![ContentPiece::Interpolation(input.parse()?)]));
        }

        if input.peek(Token![<]) {
            return Ok(Self(vec![ContentPiece::Node(input.parse()?)]));
        }

        let mut pieces = Vec::new();

        let mut content = TokenStream::new();
        while !input.is_empty() && !input.peek(Token![<]) && !input.peek(token::Brace) {
            if input.peek(token::Paren) {

            } else if input.peek(token::Bracket) {

            } // ...
            // avoided `TokenTree::Group` that can hide Brace or < in it
            else {
                let one_token = input.parse::<TokenTree>()?;
                content.extend(quote! { #one_token });
            }

            todo!()
        }

        Ok(Self(pieces))
    }
}

impl Parse for InterpolationTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(InterpolationTokens {
            _brace: syn::braced!(content in input),
            rust_expression: content.parse()?,
        })
    }
}

impl Parse for AttributeTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let value: AttributeValueTokens = input.parse()?;
        Ok(AttributeTokens { name, _eq, value })
    }
}

impl Parse for AttributeValueTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(LitStr) {
            Ok(AttributeValueTokens::StringLiteral(input.parse()?))
        } else if input.peek(token::Brace) {
            Ok(AttributeValueTokens::Interpolation(input.parse()?))
        } else {
            Err(input.error("Expected string literal or interpolation"))
        }
    }
}
