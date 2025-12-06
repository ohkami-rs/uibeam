pub(super) mod server;
#[cfg(feature = "client")]
pub(super) mod hydrate;

use super::parse::{
    AttributeTokens, AttributeValueToken, AttributeValueTokens, ContentPieceTokens, Directive,
    HtmlIdent, InterpolationTokens, NodeTokens,
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Ident, Type, spanned::Spanned};

struct Component<'n> {
    name: &'n Ident,
    attributes: &'n [AttributeTokens],
    content: Option<&'n [ContentPieceTokens]>,
}
impl NodeTokens {
    fn as_beam(&self) -> Option<Component<'_>> {
        fn as_component_name(html_ident: &HtmlIdent) -> Option<&Ident> {
            html_ident.as_ident().and_then(|ident| {
                ident
                    .to_string()
                    .chars()
                    .next()
                    .unwrap()
                    .is_ascii_uppercase()
                    .then_some(ident)
            })
        }
        match self {
            NodeTokens::EnclosingTag {
                tag,
                attributes,
                content,
                ..
            } => as_component_name(tag).map(|name| Component {
                name,
                attributes,
                content: Some(content),
            }),
            NodeTokens::SelfClosingTag {
                tag, attributes, ..
            } => as_component_name(tag).map(|name| Component {
                name,
                attributes,
                content: None,
            }),
            _ => None,
        }
    }
}

impl Component<'_> {
    fn into_rendering_expr_with(self, directives: &[Directive]) -> syn::Result<syn::Expr> {
        let Component {
            name,
            attributes,
            content,
        } = self;

        let attributes = attributes
            .iter()
            .map(|a| {
                let name = a.name.as_ident().ok_or_else(|| {
                    syn::Error::new(
                        a.name.span(),
                        "expected a valid Rust identifier for Beam property name",
                    )
                })?;
                let (value, is_literal) = match &a.value {
                    None => (quote! {true}, true),
                    Some(AttributeValueTokens { value, .. }) => match value {
                        AttributeValueToken::StringLiteral(lit) => (lit.into_token_stream(), true),
                        AttributeValueToken::IntegerLiteral(lit) => (lit.into_token_stream(), true),
                        AttributeValueToken::Interpolation(InterpolationTokens {
                            rust_expression,
                            ..
                        }) => (rust_expression.into_token_stream(), false),
                    },
                };
                Ok(if is_literal {
                    quote! {
                        #[allow(unused_braces)]
                        #name: (#value).into(),
                    }
                } else {
                    quote! {
                        #name: #value,
                    }
                })
            })
            .collect::<syn::Result<Vec<_>>>()?;

        let children = match content {
            None => None,
            Some(c) => Some({
                let children_tokens = c
                    .iter()
                    .map(ToTokens::to_token_stream)
                    .collect::<TokenStream>();
                // Explicitly using `expand()`, instead of just returning
                // `children: UI! { #(#directives)* #children_tokens }`,
                // to avoid recursive macro expansions.
                let children_tokens = crate::ui::expand(quote![
                    #(#directives)*
                    #children_tokens
                ])?;
                quote! {
                    children: #children_tokens,
                }
            }),
        };

        let render_method = if directives.iter().any(|d| d.client()) {
            quote! { ::uibeam::render_in_island }
        } else {
            quote! { ::uibeam::render_on_server }
        };

        syn::parse2(quote! {
            #render_method(#name {
                #(#attributes)*
                #children
            })
        })
    }
}

fn prop_for_event(event: &str) -> syn::Result<(Ident, Type)> {
    macro_rules! preact_handlers {
        ($($eventname:literal: $propName:ident($Event:ty);)*) => {
            match event {
                $(
                    $eventname => Ok((
                        Ident::new(stringify!($propName), proc_macro2::Span::call_site()),
                        syn::parse_quote! {::uibeam::client::web_sys::$Event}
                    )),
                )*
                _ => Err(syn::Error::new(Span::call_site(), format!(
                    "unknown event handler `on{event}`. \
                    NOTE: UIBeam's event handlers are named as `on{{event}}` with totally lowercase, \
                    e.g. `onclick` `onpointerdown`, as HTML standard. \
                    if `{event}` is a valid event, feel free to submit an issue for it: https://github.com/ohkami-rs/uibeam/issues"
                )))
            }
        };
    }
    preact_handlers! {
        "afterprint":       onAfterPrint(Event);
        "beforeprint":      onBeforePrint(Event);
        "beforeunload":     onBeforeUnload(Event);
        "beforematch":      onBeforeMatch(Event);
        "change":           onChange(Event);
        "fullscreenchange": onFullScreenChange(Event);
        "fullscreenerror":  onFullScreenError(Event);
        "load":             onLoad(Event);
        "scroll":           onScroll(Event);
        "scrollend":        onScrollEnd(Event);
        "offline":          onOffline(Event);
        "online":           onOnline(Event);

        "animationcancel":    onAnimationCancel(AnimationEvent);
        "animationend":       onAnimationEnd(AnimationEvent);
        "animationiteration": onAnimationIteration(AnimationEvent);
        "animationstart":     onAnimationStart(AnimationEvent);

        "copy":  onCopy(ClipboardEvent);
        "cut":   onCut(ClipboardEvent);
        "paste": onPaste(ClipboardEvent);

        "compositionend":    onCompositionEnd(CompositionEvent);
        "compositionstart":  onCompositionStart(CompositionEvent);
        "compositionupdate": onCompositionUpdate(CompositionEvent);

        "blur":     onBlur(FocusEvent);
        "focus":    onFocus(FocusEvent);
        "focusin":  onFocusIn(FocusEvent);
        "focusout": onFocusOut(FocusEvent);

        "input":       onInput(InputEvent);
        "beforeinput": onBeforeInput(InputEvent);

        "keydown":  onKeyDown(KeyboardEvent);
        "keyup":    onKeyUp(KeyboardEvent);

        "auxclick":    onAuxClick(MouseEvent);
        "contextmenu": onContextMenu(MouseEvent);
        "dblclick":    onDblClick(MouseEvent);
        "mousedown":   onMouseDown(MouseEvent);
        "mouseenter":  onMouseEnter(MouseEvent);
        "mouseleave":  onMouseLeave(MouseEvent);
        "mousemove":   onMouseMove(MouseEvent);
        "mouseout":    onMouseOut(MouseEvent);
        "mouseover":   onMouseOver(MouseEvent);
        "mouseup":     onMouseUp(MouseEvent);

        "click":              onClick(PointerEvent);
        "gotpointercapture":  onGotPointerCapture(PointerEvent);
        "lostpointercapture": onLostPointerCapture(PointerEvent);
        "pointercancel":      onPointerCancel(PointerEvent);
        "pointerdown":        onPointerDown(PointerEvent);
        "pointerenter":       onPointerEnter(PointerEvent);
        "pointerleave":       onPointerLeave(PointerEvent);
        "pointermove":        onPointerMove(PointerEvent);
        "pointerout":         onPointerOut(PointerEvent);
        "pointerover":        onPointerOver(PointerEvent);
        "pointerrawupdate":   onPointerRawUpdate(PointerEvent);
        "pointerup":          onPointerUp(PointerEvent);

        "touchcancel": onTouchCancel(TouchEvent);
        "touchend":    onTouchEnd(TouchEvent);
        "touchmove":   onTouchMove(TouchEvent);
        "touchstart":  onTouchStart(TouchEvent);

        "transitioncancel": onTransitionCancel(TransitionEvent);
        "transitionend":    onTransitionEnd(TransitionEvent);
        "transitionrun":    onTransitionRun(TransitionEvent);
        "transitionstart":  onTransitionStart(TransitionEvent);

        "resize": onResize(UiEvent);

        "wheel": onWheel(WheelEvent);
    }
}
