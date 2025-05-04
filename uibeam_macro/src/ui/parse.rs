use quote::{quote, ToTokens};
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

impl ContentPieceTokens {
    pub(super) fn restore(&self) -> proc_macro2::TokenStream {
        match self {
            ContentPieceTokens::Interpolation(interpolation) => interpolation.rust_expression.to_token_stream(),
            ContentPieceTokens::StaticText(lit_str) => lit_str.to_token_stream(),
            ContentPieceTokens::Node(node) => node.restore(),
        }
    }
}

impl NodeTokens {
    pub(super) fn restore(&self) -> proc_macro2::TokenStream {
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
                let attributes = attributes.iter().map(|attr| attr.restore());
                let content = content.iter().map(ContentPieceTokens::restore);
                quote! {
                    #_start_open #tag #(#attributes)* #_start_close
                    #(#content)*
                    #_end_open #_slash #_tag #_end_close
                }
            }
            NodeTokens::SelfClosingTag {
                _open,
                tag,
                attributes,
                _slash,
                _end,
            } => {
                let attributes = attributes.iter().map(|attr| attr.restore());
                quote! {
                    #_open #tag #(#attributes)* #_slash #_end
                }
            }
            NodeTokens::TextNode(pieces) => {
                let pieces = pieces.iter().map(ContentPieceTokens::restore);
                quote! {
                    #(#pieces)*
                }
            }
        }
    }
}

impl AttributeTokens {
    pub(super) fn restore(&self) -> proc_macro2::TokenStream {
        let name = self.name.restore();
        let _eq = &self._eq;
        let value = self.value.restore();
        quote! {
            #name #_eq #value
        }
    }
}
impl AttributeNameTokens {
    pub(super) fn restore(&self) -> proc_macro2::TokenStream {
        self.0
            .pairs()
            .map(|pair| {
                let section = match pair.value() {
                    AttributeNameToken::Ident(ident) => ident.to_token_stream(),
                    AttributeNameToken::Keyword(keyword) => keyword.clone(),
                };
                syn::punctuated::Pair::new(section, pair.punct().cloned())
            })
            .collect::<Punctuated<proc_macro2::TokenStream, _>>()
            .into_token_stream()
    }
}
impl AttributeValueTokens {
    pub(super) fn restore(&self) -> proc_macro2::TokenStream {
        match self {
            AttributeValueTokens::StringLiteral(lit_str) => lit_str.to_token_stream(),
            AttributeValueTokens::Interpolation(InterpolationTokens { _brace, rust_expression }) => {
                let mut restored = proc_macro2::TokenStream::new();
                _brace.surround(&mut restored, |t| t.extend(rust_expression.to_token_stream()));
                restored
            }
        }
    }
}
