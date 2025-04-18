use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{token, Token, Ident, Expr, LitStr};
use syn::punctuated::Punctuated;

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
        content: Vec<ContentPieceTokens>,
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
    TextNode(Vec<ContentPieceTokens>),
}

pub(super) enum ContentPieceTokens {
    Interpolation(InterpolationTokens),
    StaticText(LitStr),
    Node(NodeTokens),
}

pub(super) struct InterpolationTokens {
    pub(super) _brace: token::Brace,
    pub(super) rust_expression: Expr,
}

pub(super) struct AttributeTokens {
    pub(super) name: AttributeNameTokens,
    pub(super) _eq: Token![=],
    pub(super) value: AttributeValueTokens,
}

pub(super) struct AttributeNameTokens(
    /// supporting hyphenated identifiers like `data-foo`
    Punctuated<Ident, Token![-]>,
);
impl AttributeNameTokens {
    pub(super) fn span(&self) -> proc_macro2::Span {
        self.0.span()
    }
}
impl std::fmt::Display for AttributeNameTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("-")
        )
    }
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
        if input.peek(Token![<]) {
            // reject empty tags (`<>`) or end tags (`</name>`)
            if !input.peek2(Ident) {
                return Err(input.error("Expected a tag name after '<' for a start tag"));
            }

            let _start_open: Token![<] = input.parse()?;
            let tag: Ident = input.parse()?;

            let mut attributes = Vec::new();
            while let Ok(attribute) = input.parse::<AttributeTokens>() {
                attributes.push(attribute);
            }

            if input.peek(Token![/]) {
                let _slash: Token![/] = input.parse()?;
                let _end: Token![>] = input.parse()?;

                Ok(NodeTokens::SelfClosingTag {
                    _open: _start_open,
                    tag,
                    attributes,
                    _slash,
                    _end,
                })

            } else if input.peek(Token![>]) {
                let _start_close: Token![>] = input.parse()?;

                let (mut content, mut err) = (Vec::new(), None);
                while !input.is_empty() {
                    match input.parse::<ContentPieceTokens>() {
                        Ok(content_piece_tokens) => content.push(content_piece_tokens),
                        Err(e) => {err = Some(e); break}
                    }
                }

                // for better error messages
                if !input.peek(Token![<]) {
                    return Err(
                        if err.as_ref().is_some_and(|e| e.to_string().contains("expression")) {
                            err.unwrap()
                        } else {
                            input.error(format!(
                                "Unexpected {}: expected one of `</{tag}>`, another start tag, `{{expression}}`, or string literal",
                                input.cursor()
                                    .token_tree()
                                    .map(|(tt, _)| format!("`{tt}`"))
                                    .unwrap_or_else(|| "end of input".to_string())
                            ))
                        }
                    );
                }

                let _end_open: Token![<] = input.parse()?;
                let _slash: Token![/] = input.parse()?;

                let _tag: Ident = input.parse()?;
                if _tag != tag {
                    return Err(input.error(format!(
                        "Expected </{}> but found </{}>",
                        tag, input.parse::<Ident>()?.to_string()
                    )));
                }

                let _end_close: Token![>] = input.parse()?;

                Ok(NodeTokens::EnclosingTag {
                    _start_open,
                    tag,
                    attributes,
                    _start_close,
                    content,
                    _end_open,
                    _slash,
                    _tag,
                    _end_close,
                })

            } else {
                Err(input.error("Expected '>' or '/>' at the end of a tag"))
            }

        } else {
            let mut pieces = Vec::new();
            while let Ok(content_piece_tokens) = input.parse::<ContentPieceTokens>() {
                pieces.push(content_piece_tokens);
            }
            Ok(NodeTokens::TextNode(pieces))
        }
    }
}

impl Parse for ContentPieceTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(token::Brace) {
            Ok(Self::Interpolation(input.parse()?))

        } else if input.peek(LitStr) {
            Ok(Self::StaticText(input.parse()?))

        } else if input.peek(Token![<]) {
            Ok(Self::Node(input.parse()?))

        } else {
            Err(input.error("Expected one of: start tag, `{{expression}}`, or string literal"))
        }
    }
}

impl Parse for InterpolationTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut content;
        {
            syn::braced!(content in input.fork());
            if !Expr::peek(&content) {
                return Err(input.error("Expected a Rust expression inside the braces"));
            }
        }
        Ok(InterpolationTokens {
            _brace: syn::braced!(content in input),
            rust_expression: content.parse()?,
        })
    }
}

impl Parse for AttributeTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: AttributeNameTokens = input.parse()?;
        let _eq: Token![=] = input.parse()?;
        let value: AttributeValueTokens = input.parse()?;
        Ok(AttributeTokens { name, _eq, value })
    }
}

impl Parse for AttributeNameTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name = Punctuated::new();
        while input.peek(Ident) {
            name.push_value(input.parse()?);
            if !input.peek(Token![-]) {
                break;
            }
            name.push_punct(input.parse()?);
        }

        if name.is_empty() {
            return Err(input.error("Expected an identifier for the attribute name"));
        }

        Ok(Self(name))
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
