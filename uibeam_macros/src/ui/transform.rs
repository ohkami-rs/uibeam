pub(super) mod native;

#[cfg(feature = "laser")]
pub(super) mod wasm32;

use super::parse::{NodeTokens, ContentPieceTokens, HtmlIdent, AttributeTokens};
use syn::{Ident, Type};
use quote::quote;

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

fn prop_for_event(event: &str) -> Option<(Ident, Type)> {
    macro_rules! preact_handlers {
        ($($eventname:literal: $propName:ident($Event:ty);)*) => {
            match event {
                $(
                    $eventname => Some((
                        Ident::new(stringify!($propName), proc_macro2::Span::call_site()),
                        syn::parse2::<Type>(quote! {::uibeam::laser::web_sys::$Event}).unwrap()
                    )),
                )*
                _ => None
            }
        };
    }
    preact_handlers! {
        "animationcancel":    onAnimationCancel(AnimationEvent);
        "animationend":       onAnimationEnd(AnimationEvent);
        "animationiteration": onAnimationIteration(AnimationEvent);
        "animationstart":     onAnimationStart(AnimationEvent);

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

        "beforeinput": onBeforeInput(InputEvent);

        "blur":     onBlur(FocusEvent);
        "focus":    onFocus(FocusEvent);
        "focusin":  onFocusIn(FocusEvent);
        "focusout": onFocusOut(FocusEvent);

        "compositionend":    onCompositionEnd(CompositionEvent);
        "compositionstart":  onCompositionStart(CompositionEvent);
        "compositionupdate": onCompositionUpdate(CompositionEvent);

        "keydown":  onKeyDown(KeyboardEvent);
        "keypress": onKeyPress(KeyboardEvent);
        "keyup":    onKeyUp(KeyboardEvent);

        "touchcancel": onTouchCancel(TouchEvent);
        "touchend":    onTouchEnd(TouchEvent);
        "touchmove":   onTouchMove(TouchEvent);
        "touchstart":  onTouchStart(TouchEvent);

        "transitioncancel": onTransitionCancel(TransitionEvent);
        "transitionend":    onTransitionEnd(TransitionEvent);
        "transitionrun":    onTransitionRun(TransitionEvent);
        "transitionstart":  onTransitionStart(TransitionEvent);

        "wheel": onWheel(WheelEvent);

        "beforematch":      onBeforeMatch(Event);
        "change":           onChange(Event);
        "fullscreenchange": onFullScreenChange(Event);
        "fullscreenerror":  onFullScreenError(Event);
        "input":            onInput(Event);
        "load":             onLoad(Event);
        "scroll":           onScroll(Event);
        "scrollend":        onScrollEnd(Event);

        "afterprint":   onAfterPrint(Event);
        "beforeprint":  onBeforePrint(Event);
        "beforeunload": onBeforeUnload(Event);
        "offline":      onOffline(Event);
        "online":       onOnline(Event);

        "resize": onResize(UiEvent);
    }
}
