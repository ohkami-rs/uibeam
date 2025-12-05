pub(super) mod browser;
pub(super) mod server;

use super::parse::{AttributeTokens, ContentPieceTokens, HtmlIdent, NodeTokens};
use proc_macro2::Span;
use syn::{Ident, Type};

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
                    "Handler for unknown event `{event}`. If it's valid event, \
                    please submit an issue at https://github.com/ohkami-rs/uibeam/issues \
                    to add support for it! \
                    NOTE: custom event handlers are not supported in current version."
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
