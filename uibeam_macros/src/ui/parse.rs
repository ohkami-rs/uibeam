use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Ident, LitInt, LitStr, Token, token};

/// Parsed representation of the UI macro input.
///
/// This is almost HTML syntax, but with optional `@directive;`s and some Rust expressions embedded within `{}`.
pub(super) struct UITokens {
    pub(super) directives: Vec<Directive>,
    pub(super) nodes: Vec<NodeTokens>,
}

pub(super) struct Directive {
    pub(super) _at: Token![@],
    pub(super) name: Ident,
    pub(super) _semi: Token![;],
}
impl Directive {
    pub(super) fn client(&self) -> bool {
        self.name == "client"
    }
    pub(super) fn new(name: &str) -> Self {
        Directive {
            _at: Default::default(),
            name: quote::format_ident!("{name}"),
            _semi: Default::default(),
        }
    }
}

#[derive(Clone)]
pub(super) enum NodeTokens {
    Doctype {
        _open: Token![<],
        _bang: Token![!],
        _doctype: keyword::DOCTYPE,
        _html: keyword::html,
        _end: Token![>],
    },
    EnclosingTag {
        _start_open: Token![<],
        tag: HtmlIdent,
        attributes: Vec<AttributeTokens>,
        _start_close: Token![>],
        content: Vec<ContentPieceTokens>,
        _end_open: Token![<],
        _slash: Token![/],
        _tag: HtmlIdent,
        _end_close: Token![>],
    },
    SelfClosingTag {
        _open: Token![<],
        tag: HtmlIdent,
        attributes: Vec<AttributeTokens>,
        _slash: Token![/],
        _end: Token![>],
    },
    TextNode(Vec<ContentPieceTokens>),
}

mod keyword {
    syn::custom_keyword!(DOCTYPE);
    syn::custom_keyword!(html);
}

#[derive(Clone)]
pub(super) struct HtmlIdent {
    head: Ident,
    rest: Vec<(Token![-], Ident)>,
}
impl HtmlIdent {
    pub(super) fn as_ident(&self) -> Option<&Ident> {
        self.rest.is_empty().then_some(&self.head)
    }
}
impl PartialEq for HtmlIdent {
    fn eq(&self, other: &Self) -> bool {
        self.head == other.head
            && self.rest.len() == other.rest.len()
            && Iterator::zip(self.rest.iter(), other.rest.iter()).all(|((_, a), (_, b))| a == b)
    }
}
impl std::fmt::Display for HtmlIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.head)?;
        for (_, ident) in &self.rest {
            write!(f, "-{ident}")?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(super) enum ContentPieceTokens {
    Interpolation(InterpolationTokens),
    StaticText(LitStr),
    Node(NodeTokens),
}

#[derive(Clone)]
pub(super) struct InterpolationTokens {
    pub(super) _unsafe: Option<Token![unsafe]>,
    pub(super) _brace: token::Brace,
    pub(super) rust_expression: Expr,
}

#[derive(Clone)]
pub(super) struct AttributeTokens {
    pub(super) name: HtmlIdent,
    pub(super) value: Option<AttributeValueTokens>,
}

#[derive(Clone)]
pub(super) struct AttributeValueTokens {
    pub(super) _eq: Token![=],
    pub(super) value: AttributeValueToken,
}
#[derive(Clone)]
pub(super) enum AttributeValueToken {
    StringLiteral(LitStr),
    IntegerLiteral(LitInt),
    Interpolation(InterpolationTokens),
}

//////////////////////////////////////////////////////////////////////////////////////////

impl Parse for UITokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut directives = Vec::new();
        while input.peek(Token![@]) {
            directives.push(input.parse()?);
        }

        let mut nodes = Vec::new();
        while !input.is_empty() {
            nodes.push(input.parse()?);
        }

        Ok(UITokens { directives, nodes })
    }
}

impl Parse for Directive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _at: Token![@] = input.parse()?;
        let name: Ident = input.parse()?;
        let _semi: Token![;] = input.parse()?;
        Ok(Directive { _at, name, _semi })
    }
}

impl Parse for NodeTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) {
            if input.peek2(Token![!]) {
                let _open: Token![<] = input.parse()?;
                let _bang: Token![!] = input.parse()?;
                let _doctype: keyword::DOCTYPE = input.parse()?;
                let _html: keyword::html = input.parse()?;
                let _end: Token![>] = input.parse()?;

                return Ok(NodeTokens::Doctype {
                    _open,
                    _bang,
                    _doctype,
                    _html,
                    _end,
                });
            }

            // reject empty tags (`<>`) or end tags (`</name>`)
            if !input.peek2(Ident) {
                return Err(input.error("Expected a tag name after '<' for a start tag"));
            }

            let _start_open: Token![<] = input.parse()?;
            let tag: HtmlIdent = input.parse()?;

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
                if tag.head == "br" || tag.head == "meta" || tag.head == "link" || tag.head == "hr"
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
                #[allow(clippy::nonminimal_bool)]
                while (!input.is_empty()) && !(input.peek(Token![<]) && input.peek2(Token![/])) {
                    content.push(input.parse()?);
                }

                let _end_open: Token![<] = input.parse()?;
                let _slash: Token![/] = input.parse()?;

                let _tag: HtmlIdent = input.parse()?;
                if _tag != tag {
                    return Err(syn::Error::new(
                        tag.span(),
                        format!("Not closing tag: no corresponded `/>` or `</{tag}>` exists"),
                    ));
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
        if input.peek(Token![unsafe]) || input.peek(token::Brace) {
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
        let _unsafe = input
            .peek(Token![unsafe])
            .then(|| input.parse())
            .transpose()?;

        let mut content;
        {
            syn::braced!(content in input.fork());
            if !Expr::peek(&content) {
                return Err(input.error("Expected a Rust expression inside the braces"));
            }
        }

        Ok(InterpolationTokens {
            _unsafe,
            _brace: syn::braced!(content in input),
            rust_expression: content.parse()?,
        })
    }
}

impl Parse for AttributeTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: HtmlIdent = input.parse()?;
        let value: Option<AttributeValueTokens> =
            input.peek(Token![=]).then(|| input.parse()).transpose()?;
        Ok(AttributeTokens { name, value })
    }
}
impl Parse for HtmlIdent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let head = syn::ext::IdentExt::parse_any(input)?;

        let mut rest = vec![];
        while input.peek(Token![-]) {
            let hyphen: Token![-] = input.parse()?;
            let ident = syn::ext::IdentExt::parse_any(input)?;
            rest.push((hyphen, ident));
        }

        Ok(Self { head, rest })
    }
}
impl Parse for AttributeValueTokens {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _eq: Token![=] = input.parse()?;
        let value = if input.peek(LitStr) {
            AttributeValueToken::StringLiteral(input.parse()?)
        } else if input.peek(LitInt) {
            AttributeValueToken::IntegerLiteral(input.parse()?)
        } else if input.peek(token::Brace) {
            // NOT expect `unsafe` here
            AttributeValueToken::Interpolation(input.parse()?)
        } else {
            return Err(input.error("Expected string literal or interpolation"));
        };
        Ok(AttributeValueTokens { _eq, value })
    }
}

//////////////////////////////////////////////////////////////////////////////////////////

impl ToTokens for Directive {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        self._at.to_tokens(t);
        self.name.to_tokens(t);
        self._semi.to_tokens(t);
    }
}

impl ToTokens for ContentPieceTokens {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        match self {
            ContentPieceTokens::Node(node) => node.to_tokens(t),
            ContentPieceTokens::StaticText(lit_str) => lit_str.to_tokens(t),
            ContentPieceTokens::Interpolation(interpolation) => interpolation.to_tokens(t),
        }
    }
}

impl ToTokens for InterpolationTokens {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        if let Some(_unsafe) = &self._unsafe {
            _unsafe.to_tokens(t);
        }
        self._brace.surround(t, |inner| {
            self.rust_expression.to_tokens(inner);
        });
    }
}

impl ToTokens for NodeTokens {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        match self {
            NodeTokens::Doctype {
                _open,
                _bang,
                _doctype,
                _html,
                _end,
            } => {
                (quote! {
                    #_open #_bang #_doctype #_html #_end
                })
                .to_tokens(t);
            }
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
                let attributes = attributes.iter().map(AttributeTokens::to_token_stream);
                let content = content.iter().map(ContentPieceTokens::to_token_stream);
                (quote! {
                    #_start_open #tag #(#attributes)* #_start_close
                    #(#content)*
                    #_end_open #_slash #_tag #_end_close
                })
                .to_tokens(t);
            }
            NodeTokens::SelfClosingTag {
                _open,
                tag,
                attributes,
                _slash,
                _end,
            } => {
                let attributes = attributes.iter().map(AttributeTokens::to_token_stream);
                (quote! {
                    #_open #tag #(#attributes)* #_slash #_end
                })
                .to_tokens(t);
            }
            NodeTokens::TextNode(pieces) => {
                let pieces = pieces.iter().map(ContentPieceTokens::to_token_stream);
                (quote! {
                    #(#pieces)*
                })
                .to_tokens(t);
            }
        }
    }
}

impl ToTokens for HtmlIdent {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        self.head.to_tokens(t);
        for (hyphen, ident) in &self.rest {
            hyphen.to_tokens(t);
            ident.to_tokens(t);
        }
    }
}

impl ToTokens for AttributeTokens {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        self.name.to_tokens(t);
        if let Some(value) = &self.value {
            value.to_tokens(t);
        }
    }
}

impl ToTokens for AttributeValueTokens {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        self._eq.to_tokens(t);
        self.value.to_tokens(t);
    }
}
impl ToTokens for AttributeValueToken {
    fn to_tokens(&self, t: &mut proc_macro2::TokenStream) {
        match self {
            AttributeValueToken::StringLiteral(lit_str) => {
                lit_str.to_tokens(t);
            }
            AttributeValueToken::IntegerLiteral(lit_int) => {
                LitStr::new(lit_int.base10_digits(), lit_int.span()).to_tokens(t);
            }
            AttributeValueToken::Interpolation(InterpolationTokens {
                _unsafe,
                _brace,
                rust_expression,
            }) => {
                assert!(_unsafe.is_none());
                _brace.surround(t, |inner| rust_expression.to_tokens(inner));
            }
        }
    }
}
