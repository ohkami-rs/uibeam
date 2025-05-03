use quote::ToTokens;
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
pub(super) struct Beam<'n> {
    pub(super) tag: &'n Ident,
    pub(super) attributes: &'n [AttributeTokens],
    pub(super) content: Option<&'n [ContentPieceTokens]>,
}
impl NodeTokens {
    pub(super) fn as_beam(&self) -> Option<Beam<'_>> {
        let is_beam_ident = |ident: &Ident| {
            ident.to_string().chars().next().unwrap().is_ascii_uppercase()
        };
        match self {
            NodeTokens::EnclosingTag { tag, attributes, content, .. } => {
                if is_beam_ident(tag) {
                    Some(Beam {
                        tag,
                        attributes,
                        content: Some(content),
                    })
                } else {
                    None
                }
            }
            NodeTokens::SelfClosingTag { tag, attributes, .. } => {
                if is_beam_ident(tag) {
                    Some(Beam {
                        tag,
                        attributes,
                        content: None,
                    })
                } else {
                    None
                }
            }
            NodeTokens::TextNode(_) => None,
        }
    }
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
    Punctuated<AttributeNameToken, Token![-]>,
);
enum AttributeNameToken {
    Ident(Ident),
    // support Rust keywords as attribute names like `<input type="text" for="foo" />`
    Keyword(proc_macro2::TokenStream),
}
impl AttributeNameTokens {
    pub(super) fn as_ident(&self) -> Option<Ident> {
        (self.0.len() == 1).then_some(match self.0.first().unwrap() {
            AttributeNameToken::Ident(ident) => ident.clone(),
            AttributeNameToken::Keyword(keyword) => Ident::new_raw(&keyword.to_string(), keyword.span()),
        })
    }
}
impl std::fmt::Display for AttributeNameTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0
            .iter()
            .map(|token| match token {
                AttributeNameToken::Ident(ident) => ident.to_string(),
                AttributeNameToken::Keyword(keyword) => keyword.to_string(),
            })
            .collect::<Vec<_>>()
            .join("-")
        )
    }
}

pub(super) enum AttributeValueTokens {
    StringLiteral(LitStr),
    Interpolation(InterpolationTokens),
}

//////////////////////////////////////////////////////////////////////////////////////////

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

                // tolerantly accept some self-closing tags without a slash
                if tag == "br"
                || tag == "meta"
                || tag == "link"
                || tag == "hr"
                {
                    return Ok(NodeTokens::SelfClosingTag {
                        _open: _start_open,
                        tag,
                        attributes,
                        _slash: Token![/](input.span()),
                        _end: _start_close,
                    });
                }

                let mut content = Vec::<ContentPieceTokens>::new();
                while (!input.is_empty()) && !(input.peek(Token![<]) && input.peek2(Token![/])) {
                    content.push(input.parse()?);
                }

                let _end_open: Token![<] = input.parse()?;
                let _slash: Token![/] = input.parse()?;

                let _tag: Ident = input.parse()?;
                if _tag != tag {
                    return Err(syn::Error::new(tag.span(), format!("Not closing tag: no corresponded `/>` or `</{tag}>` exists")))
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
            Err(input.error("Expected one of: start tag, string literal, {expression}"))
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

        macro_rules! push_ident_or_keyword {
            ($($keyword:tt)*) => {
                if input.peek(Ident) {
                    name.push_value(AttributeNameToken::Ident(input.parse()?));
                }
                $(
                    else if input.peek(Token![$keyword]) {
                        name.push_value(AttributeNameToken::Keyword(input.parse::<Token![$keyword]>()?.into_token_stream()));
                    }
                )*
                else {
                    break;
                }
            };
        }

        loop {
            push_ident_or_keyword![
                abstract as async await
                become box break
                const continue crate
                do dyn
                else enum extern
                final fn for
                // gen
                if impl in
                let loop
                macro match mod move
                override
                priv pub
                ref return
                self Self static struct super
                trait type typeof try
                unsafe unsized use
                virtual
                where while
                yield
            ];

            if input.peek(Token![-]) {
                name.push_punct(input.parse()?);
            } else {
                break;
            }
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

//////////////////////////////////////////////////////////////////////////////////////////

fn collect_span(
    iter: impl Iterator<Item = proc_macro2::Span>,
) -> Option<proc_macro2::Span> {
    iter.reduce(|s1, s2| joined_span!(s1, s2))
}

impl NodeTokens {
    pub(super) fn span(&self) -> Option<proc_macro2::Span> {
        match self {
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
                Some(joined_span!(
                    _start_open.span(),
                    tag.span(),
                    collect_span(attributes.iter().flat_map(AttributeTokens::span)),
                    _start_close.span(),
                    collect_span(content.iter().flat_map(ContentPieceTokens::span)),
                    _end_open.span(),
                    _slash.span(),
                    _tag.span(),
                    _end_close.span(),
                ))
            }
            NodeTokens::SelfClosingTag {
                _open,
                tag,
                attributes,
                _slash,
                _end,
            } => {
                Some(joined_span!(
                    _open.span(),
                    tag.span(),
                    collect_span(attributes.iter().flat_map(AttributeTokens::span)),
                    _slash.span(),
                    _end.span(),
                ))
            }
            NodeTokens::TextNode(pieces) => {
                collect_span(pieces.iter().flat_map(ContentPieceTokens::span))
            }
        }
    }
}

impl ContentPieceTokens {
    pub(super) fn span(&self) -> Option<proc_macro2::Span> {
        match self {
            ContentPieceTokens::Interpolation(interpolation) => interpolation.span(),
            ContentPieceTokens::StaticText(lit_str) => Some(lit_str.span()),
            ContentPieceTokens::Node(node) => node.span(),
        }
    }
}

impl InterpolationTokens {
    pub(super) fn span(&self) -> Option<proc_macro2::Span> {
        Some(self._brace.span.span())
    }
}

impl AttributeTokens {
    pub(super) fn span(&self) -> Option<proc_macro2::Span> {
        Some(joined_span!(
            self.name.span()?,
            self._eq.span(),
            self.value.span(),
        ))
    }
}
impl AttributeNameTokens {
    pub(super) fn span(&self) -> Option<proc_macro2::Span> {
        Some(self.0
            .pairs()
            .map(|token_punct| {
                let token_span = match token_punct.value() {
                    AttributeNameToken::Ident(ident) => ident.span(),
                    AttributeNameToken::Keyword(keyword) => keyword.span(),
                };
                match token_punct.punct() {
                    None => token_span,
                    Some(p) => joined_span!(token_span, p.span()),
                }
            })
            .reduce(|s1, s2| joined_span!(s1, s2))
            .expect("AttributeNameTokens must have at least one token")
        )
    }
}
impl AttributeValueTokens {
    pub(super) fn span(&self) -> Option<proc_macro2::Span> {
        match self {
            AttributeValueTokens::StringLiteral(lit_str) => Some(lit_str.span()),
            AttributeValueTokens::Interpolation(interpolation) => interpolation.span(),
        }
    }
}
