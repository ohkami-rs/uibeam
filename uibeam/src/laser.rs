#![cfg(feature = "laser")]

// TODO: support more events (update together with `uibeam_macros/src/ui/transform.rs`)
pub use ::web_sys::{AnimationEvent, MouseEvent, PointerEvent, InputEvent, FocusEvent, CompositionEvent, KeyboardEvent, TouchEvent, TransitionEvent, WheelEvent, Event};

#[doc(hidden)]
pub use {::wasm_bindgen, ::js_sys, ::web_sys, ::serde, ::serde_wasm_bindgen};

#[doc(hidden)]
pub fn serialize_props<P: ::serde::Serialize>(props: &P) -> String {
    ::serde_json::to_string(props).unwrap()
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub trait Laser_attribute {}

pub trait Laser: Laser_attribute {
    fn render(self) -> crate::UI;
}

#[cfg(target_arch = "wasm32")]
mod preact {
    use super::*;
    
    #[wasm_bindgen(module = "preact")]
    unsafe extern "C" {
        #[wasm_bindgen(js_name = "hydrate")]
        pub(super) fn hydrate(vdom: JsValue, container: ::web_sys::Node);

        #[wasm_bindgen(js_name = "createElement")]
        pub(super) fn create_element(
            r#type: JsValue,
            props: Object,
            children: Array,
        ) -> JsValue;

        #[wasm_bindgen(js_name = "cloneElement")]
        pub(super) fn clone_element(
            vdom: JsValue,
            props: Object,
            children: Array,
        ) -> JsValue;

        #[wasm_bindgen(js_name = "createRef")]
        pub(super) fn create_ref() -> JsValue;

        #[wasm_bindgen(js_name = "Fragment")]
        pub(super) fn fragment(
            props: Object,
        ) -> JsValue;
    }

    #[wasm_bindgen(module = "@preact/signals")]
    unsafe extern "C" {
        #[wasm_bindgen(js_name = "useSignal")]
        pub(super) fn signal(value: JsValue) -> Object;

        #[wasm_bindgen(js_name = "useComputed")]
        pub(super) fn computed(f: Function) -> Object;

        #[wasm_bindgen(js_name = "useSignalEffect")]
        pub(super) fn effect(f: Function);

        #[wasm_bindgen(js_name = "batch")]
        pub(super) fn batch(f: Function);

        #[wasm_bindgen(js_name = "untracked")]
        pub(super) fn untracked(f: Function);
    }
}

#[cfg(target_arch = "wasm32")]
use {
    ::wasm_bindgen::prelude::*,
    ::js_sys::{Function, Array, Object, Reflect},
};

#[cfg(target_arch = "wasm32")]
pub fn hydrate(
    vdom: VNode,
    container: ::web_sys::Node,
) {
    preact::hydrate(vdom.0, container);
}

#[cfg(target_arch = "wasm32")]
pub struct VNode(JsValue);

#[cfg(target_arch = "wasm32")]
pub struct NodeType(JsValue);

#[cfg(target_arch = "wasm32")]
impl NodeType {
    pub fn tag(tag: &'static str) -> NodeType {
        NodeType(tag.into())
    }

    pub fn component<L>() -> NodeType
    where
        L: Laser + for<'de> serde::Deserialize<'de>,
    {
        let component_function: Function = Closure::<dyn Fn(JsValue)->JsValue>::new(|props| {
            let props: L = serde_wasm_bindgen::from_value(props).unwrap_throw();
            <L as Laser>::render(props).into_vdom().0
        }).into_js_value().unchecked_into();

        NodeType(component_function.unchecked_into())
    }
}

#[cfg(target_arch = "wasm32")]
impl VNode {
    pub fn new(
        r#type: NodeType,
        props: Object,
        children: Vec<VNode>,
    ) -> VNode {
        VNode(preact::create_element(
            r#type.0,
            props,
            children.into_iter().map(|vdom| vdom.0).collect::<Array>(),
        ))
    }

    pub fn fragment(
        children: Vec<VNode>,
    ) -> VNode {
        let props = Object::new();
        Reflect::set(
            &props,
            &"children".into(),
            &children.into_iter().map(|vdom| vdom.0).collect::<Array>(),
        ).ok();
        VNode(preact::fragment(props))
    }

    pub fn text(text: impl Into<std::borrow::Cow<'static, str>>) -> VNode {
        match text.into() {
            std::borrow::Cow::Owned(s) => VNode(s.into()),
            std::borrow::Cow::Borrowed(s) => VNode(s.into()),
        }
    }
}

pub struct Signal<T: serde::Serialize + for<'de>serde::Deserialize<'de>> {
    #[cfg(target_arch = "wasm32")]
    preact_signal: Object,
    /// buffer for `Deref` impl on single-threaded wasm
    /// (and also used for template rendering)
    current_value: std::rc::Rc<std::cell::UnsafeCell<T>>,
}

impl<T: serde::Serialize + for<'de>serde::Deserialize<'de>> Clone for Signal<T> {// not require T: Clone
    fn clone(&self) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            preact_signal: self.preact_signal.clone(),
            current_value: self.current_value.clone(),
        }
    }
}

impl<T: serde::Serialize + for<'de>serde::Deserialize<'de>> std::ops::Deref for Signal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(not(target_arch = "wasm32"))] {// for template rendering
            unsafe {&*self.current_value.get()}
        }
        #[cfg(target_arch = "wasm32")] {
            let value = serde_wasm_bindgen::from_value(
                // TODO: skip deserialization if value is not changed 
                Reflect::get(&self.preact_signal, &"value".into()).unwrap_throw()
            ).unwrap_throw();
            unsafe { *self.current_value.get() = value; }
            unsafe {&*self.current_value.get()}
        }
    }
}

impl<T: serde::Serialize + for<'de>serde::Deserialize<'de>> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            #[cfg(target_arch = "wasm32")]
            preact_signal: preact::signal(serde_wasm_bindgen::to_value(&value).unwrap_throw()),
            current_value: std::rc::Rc::new(std::cell::UnsafeCell::new(value)),
        }
    }

    pub fn set(&self, value: T) {
        #[cfg(not(target_arch = "wasm32"))] {// for template rendering
            unsafe { *self.current_value.get() = value; }
        }
        #[cfg(target_arch = "wasm32")] {
            Reflect::set(
                &self.preact_signal,
                &"value".into(),
                &serde_wasm_bindgen::to_value(&value).unwrap_throw()
            ).unwrap_throw();
        }
    }
}

pub struct Computed<T: serde::Serialize + for<'de>serde::Deserialize<'de>>(Signal<T>);

impl<T: serde::Serialize + for<'de>serde::Deserialize<'de>> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: serde::Serialize + for<'de>serde::Deserialize<'de>> std::ops::Deref for Computed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: serde::Serialize + for<'de>serde::Deserialize<'de>> Computed<T> {
    pub fn new(getter: impl (Fn() -> T) + 'static) -> Self {
        #[cfg(not(target_arch = "wasm32"))] {// for template rendering
            Self(Signal::new(getter()))
        }
        #[cfg(target_arch = "wasm32")] {
            let init = getter();

            let preact_computed = preact::computed(Closure::<dyn Fn() -> JsValue>::new(move || {
                serde_wasm_bindgen::to_value(&getter()).unwrap_throw()
            }).into_js_value().unchecked_into());

            Self(Signal {
                preact_signal: preact_computed,
                current_value: std::rc::Rc::new(std::cell::UnsafeCell::new(init)),
            })
        }
    }
}

#[macro_export]
macro_rules! callback {
    ([$($dep:ident),*], |$($arg:ident $(: $Type:ty)?),*| $result:expr) => {
        {
            $(let $dep = $dep.clone();)*
            move |$($arg $(: $Type)?),*| $result
        }
    };
    ([$($dep:ident),*], |_ $(: $Type:ty)?| $result:expr) => {
        {
            $(let $dep = $dep.clone();)*
            move |_ $(: $Type)?| $result
        }
    };
}

pub fn computed<T: serde::Serialize + for<'de>serde::Deserialize<'de> + 'static>(
    getter: impl (Fn() -> T) + 'static
) -> Computed<T> {
    Computed::new(getter)
}

#[macro_export]
macro_rules! computed {
    ($($t:tt)*) => {
        $crate::laser::computed($crate::callback!($($t)*))
    };
}

pub fn effect(
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    f: impl Fn() + 'static
) {
    #[cfg(target_arch = "wasm32")] {
        let f = Closure::<dyn Fn()>::new(f)
            .into_js_value()
            .unchecked_into();
        preact::effect(f);
    }
}

#[macro_export]
macro_rules! effect {
    ($($t:tt)*) => {
        $crate::laser::effect($crate::callback!($($t)*))
    };
}

pub fn batch(
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    f: impl Fn() + 'static
) {
    #[cfg(target_arch = "wasm32")] {
        let f = Closure::<dyn Fn()>::new(f)
            .into_js_value()
            .unchecked_into();
        preact::batch(f);
    }
}

pub fn untracked(
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    f: impl Fn() + 'static
) {
    #[cfg(target_arch = "wasm32")] {
        let f = Closure::<dyn Fn()>::new(f)
            .into_js_value()
            .unchecked_into();
        preact::untracked(f);
    }
}
