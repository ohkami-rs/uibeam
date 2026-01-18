#![cfg(feature = "client")]

mod runtime;
mod signals;

pub use signals::{Signal, Computed, Effect, EffectScope};

// TODO: support more events (update together with `uibeam_macros/src/ui/transform.rs`)
pub use ::web_sys::{
    AnimationEvent, ClipboardEvent, CompositionEvent, Event, FocusEvent, InputEvent, KeyboardEvent,
    MouseEvent, PointerEvent, TouchEvent, TransitionEvent, UiEvent, WheelEvent,
};

#[doc(hidden)]
pub use {::js_sys, ::serde, ::serde_wasm_bindgen, ::wasm_bindgen, ::web_sys};

#[doc(hidden)]
#[inline]
pub fn serialize_props<P: super::IslandBoundary>(props: &P) -> String {
    ::serde_json::to_string(props).unwrap()
}
