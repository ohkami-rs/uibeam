pub(super) mod native;

#[cfg(feature = "laser")]
pub(super) mod wasm32;

use super::parse::{NodeTokens, ContentPieceTokens, HtmlIdent, AttributeTokens};
use syn::Ident;

struct Component<'n> {
    name: &'n Ident,
    attributes: &'n [AttributeTokens],
    content: Option<&'n [ContentPieceTokens]>,
}
impl NodeTokens {
    fn as_beam(&self) -> Option<Component<'_>> {
        fn as_component_name(html_ident: &HtmlIdent) -> Option<&Ident> {
            html_ident
                .as_ident()
                .map(|ident| ident.to_string().chars().next().unwrap().is_ascii_uppercase().then_some(ident))
                .flatten()
        }
        match self {
            NodeTokens::EnclosingTag { tag, attributes, content, .. } => {
                as_component_name(tag).map(|name| Component {
                    name,
                    attributes,
                    content: Some(content),
                })
            }
            NodeTokens::SelfClosingTag { tag, attributes, .. } => {
                as_component_name(tag).map(|name| Component {
                    name,
                    attributes,
                    content: None,
                })
            }
            _ => None,
        }
    }
}
