// TODO: support more events (update together with `uibeam_macros/src/ui/transform.rs`)
pub use ::web_sys::{
    AnimationEvent, ClipboardEvent, CompositionEvent, Event, FocusEvent, InputEvent, KeyboardEvent,
    MouseEvent, PointerEvent, TouchEvent, TransitionEvent, UiEvent, WheelEvent,
};

#[doc(hidden)]
pub use {::js_sys, ::serde, ::serde_wasm_bindgen, ::wasm_bindgen, ::web_sys};

#[doc(hidden)]
#[inline]
pub fn serialize_props<P: ::serde::Serialize>(props: &P) -> String {
    ::serde_json::to_string(props).unwrap()
}

#[cfg(client)]
mod preact {
    use super::*;

    #[wasm_bindgen(module = "preact")]
    unsafe extern "C" {
        #[wasm_bindgen(js_name = "hydrate")]
        pub(super) fn hydrate(vdom: JsValue, container: ::web_sys::Node);

        #[wasm_bindgen(js_name = "createElement")]
        pub(super) fn create_element(r#type: JsValue, props: Object, children: Array) -> JsValue;

        #[wasm_bindgen(js_name = "cloneElement")]
        pub(super) fn clone_element(vdom: JsValue, props: Object, children: Array) -> JsValue;

        #[wasm_bindgen(js_name = "createRef")]
        pub(super) fn create_ref() -> JsValue;

        #[wasm_bindgen(js_name = "Fragment")]
        pub(super) fn fragment(props: Object) -> JsValue;
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

#[cfg(client)]
use {
    ::js_sys::{Array, Function, Object, Reflect},
    ::wasm_bindgen::prelude::*,
};

#[cfg(client)]
pub fn hydrate(vdom: VNode, container: ::web_sys::Node) {
    preact::hydrate(vdom.0, container);
}

#[cfg(client)]
pub struct VNode(JsValue);

#[cfg(client)]
pub struct NodeType(JsValue);

#[cfg(client)]
impl NodeType {
    pub fn tag(tag: &'static str) -> NodeType {
        NodeType(tag.into())
    }

    pub fn component<L>() -> NodeType
    where
        L: Laser + for<'de> serde::Deserialize<'de>,
    {
        let component_function: Function = Closure::<dyn Fn(JsValue) -> JsValue>::new(|props| {
            let props: L = serde_wasm_bindgen::from_value(props).unwrap_throw();
            <L as Laser>::render(props).into_vdom().0
        })
        .into_js_value()
        .unchecked_into();

        NodeType(component_function.unchecked_into())
    }
}

#[cfg(client)]
impl VNode {
    pub fn new(r#type: NodeType, props: Object, children: Vec<VNode>) -> VNode {
        VNode(preact::create_element(
            r#type.0,
            props,
            children.into_iter().map(|vdom| vdom.0).collect::<Array>(),
        ))
    }

    pub fn fragment(children: Vec<VNode>) -> VNode {
        let props = Object::new();
        Reflect::set(
            &props,
            &"children".into(),
            &children.into_iter().map(|vdom| vdom.0).collect::<Array>(),
        )
        .ok();
        VNode(preact::fragment(props))
    }

    pub fn text(text: impl Into<std::borrow::Cow<'static, str>>) -> VNode {
        match text.into() {
            std::borrow::Cow::Owned(s) => VNode(s.into()),
            std::borrow::Cow::Borrowed(s) => VNode(s.into()),
        }
    }
}

pub struct Signal<T: serde::Serialize + for<'de> serde::Deserialize<'de>> {
    #[cfg(client)]
    preact_signal: Object,
    /// buffer for `Deref` impl on single-threaded wasm
    /// (and also used for template rendering)
    current_value: std::rc::Rc<std::cell::UnsafeCell<T>>,
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> Clone for Signal<T> {
    // not require T: Clone
    fn clone(&self) -> Self {
        Self {
            #[cfg(client)]
            preact_signal: self.preact_signal.clone(),
            current_value: self.current_value.clone(),
        }
    }
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> std::ops::Deref for Signal<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(not(client))]
        {
            // for template rendering
            unsafe { &*self.current_value.get() }
        }
        #[cfg(client)]
        {
            let value = serde_wasm_bindgen::from_value(
                // TODO: skip deserialization if value is not changed
                Reflect::get(&self.preact_signal, &"value".into()).unwrap_throw(),
            )
            .unwrap_throw();
            unsafe {
                *self.current_value.get() = value;
            }
            unsafe { &*self.current_value.get() }
        }
    }
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            #[cfg(client)]
            preact_signal: preact::signal(serde_wasm_bindgen::to_value(&value).unwrap_throw()),
            current_value: std::rc::Rc::new(std::cell::UnsafeCell::new(value)),
        }
    }

    pub fn set(&self, value: T) {
        #[cfg(not(client))]
        {
            // for template rendering
            unsafe {
                *self.current_value.get() = value;
            }
        }
        #[cfg(client)]
        {
            Reflect::set(
                &self.preact_signal,
                &"value".into(),
                &serde_wasm_bindgen::to_value(&value).unwrap_throw(),
            )
            .unwrap_throw();
        }
    }
}

pub struct Computed<T: serde::Serialize + for<'de> serde::Deserialize<'de>>(Signal<T>);

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> std::ops::Deref for Computed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> Computed<T> {
    pub fn new(getter: impl (Fn() -> T) + 'static) -> Self {
        #[cfg(not(client))]
        {
            // for template rendering
            Self(Signal::new(getter()))
        }
        #[cfg(client)]
        {
            let init = getter();

            let preact_computed = preact::computed(
                Closure::<dyn Fn() -> JsValue>::new(move || {
                    serde_wasm_bindgen::to_value(&getter()).unwrap_throw()
                })
                .into_js_value()
                .unchecked_into(),
            );

            Self(Signal {
                preact_signal: preact_computed,
                current_value: std::rc::Rc::new(std::cell::UnsafeCell::new(init)),
            })
        }
    }
}

/// Shorthand for creating closures that capture variables by cloning them.
///
/// This is useful when creating **event handlers or callbacks using signals**:
///
/// ```
/// use uibeam::{Signal, callback};
/// use uibeam::laser::{InputEvent, PointerEvent};
/// use wasm_bindgen::JsCast;
/// use web_sys::HtmlInputElement;
///
/// # fn usage() {
/// let name = Signal::new("Alice".to_owned());
/// let count = Signal::new(0);
///
/// let handle_name_input = callback!([name], |e: InputEvent| {
///     let input_element: HtmlInputElement = e
///         .current_target().unwrap()
///         .dyn_into().unwrap();
///     name.set(input_element.value());
/// });
///
/// let handle_increment_click = callback!([count], |_: PointerEvent| {
///     count.set(*count + 1);
/// });
/// # }
/// ```
///
/// ## Example
/// ```
/// use uibeam::{UI, Laser, Signal, callback};
///
/// #[Laser]
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct Counter {
///     pub initial_count: i32,
/// }
///
/// impl Laser for Counter {
///     fn render(self) -> UI {
///         let count = Signal::new(self.initial_count);
///
///         let increment = callback!([count], |_| {
///             count.set(*count + 1);
///         });
///
///         let decrement = callback!([count], |_| {
///             count.set(*count - 1);
///         });
///
///         UI! {
///             <div class="w-[144px]">
///                 <p class="text-2xl font-bold text-center">
///                     "Count: "{*count}
///                 </p>
///                 <div class="text-center">
///                     <button
///                         class="cursor-pointer bg-red-500  w-[32px] py-1 text-white rounded-md"
///                         onclick={decrement}
///                     >"-"</button>
///                     <button
///                         class="cursor-pointer bg-blue-500 w-[32px] py-1 text-white rounded-md"
///                         onclick={increment}
///                     >"+"</button>
///                 </div>
///             </div>
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! callback {
    ([$($dep:ident),*], || $result:expr) => {
        {
            $(let $dep = $dep.clone();)+
            move || $result
        }
    };
    ([$($dep:ident),*], |_ $(: $Type:ty)?| $result:expr) => {
        {
            $(let $dep = $dep.clone();)*
            move |_ $(: $Type)?| $result
        }
    };
    ([$($dep:ident),*], |$($arg:ident $(: $Type:ty)?),+| $result:expr) => {
        {
            $(let $dep = $dep.clone();)+
            move |$($arg $(: $Type)?),+| $result
        }
    };
}

pub fn computed<T: serde::Serialize + for<'de> serde::Deserialize<'de> + 'static>(
    getter: impl (Fn() -> T) + 'static,
) -> Computed<T> {
    Computed::new(getter)
}

/// `computed!([deps, ...], || -> T { ... })` creates a `Computed<T>` signal
/// that automatically updates when any of the `deps` signals change.
///
/// This is shorthand for `computed(callback!(...))`.
///
/// ## Example
/// ```
/// use uibeam::{UI, Laser, Signal, callback, computed};
///
/// #[Laser]
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct ComputedExample;
///
/// impl Laser for ComputedExample {
///     fn render(self) -> UI {
///         let count = Signal::new(0u32);
///
///         let count_squared = computed!([count], || {
///             (*count).pow(2)
///         });
///
///         let increment = callback!([count], |_| {
///             count.set(*count + 1);
///         });
///
///         UI! {
///             <div class="w-[144px]">
///                 <p class="text-2xl font-bold text-center">
///                     "Count: "{*count}
///                 </p>
///                 <p class="text-2xl font-bold text-center">
///                     "Count Squared: "{*count_squared}
///                 </p>
///                 <div class="text-center">
///                     <button
///                         class="cursor-pointer bg-blue-500 w-[32px] py-1 text-white rounded-md"
///                         onclick={increment}
///                     >"+"</button>
///                 </div>
///             </div>
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! computed {
    ($($t:tt)*) => {
        $crate::laser::computed($crate::callback!($($t)*))
    };
}

pub fn effect(#[cfg_attr(not(client), allow(unused_variables))] f: impl Fn() + 'static) {
    #[cfg(client)]
    {
        let f = Closure::<dyn Fn()>::new(f).into_js_value().unchecked_into();
        preact::effect(f);
    }
}

/// `effect!([deps, ...], || { ... })` creates a reactive effect that automatically
/// re-runs whenever any of the `deps` signals change.
///
/// This is shorthand for `effect(callback!(...))`.
///
/// ## Example
/// ```
/// use uibeam::{UI, Laser, Signal, callback, effect};
/// use web_sys::console;
///
/// #[Laser]
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct EffectExample;
///
/// impl Laser for EffectExample {
///     fn render(self) -> UI {
///         let count = Signal::new(0);
///
///         effect!([count], || {
///             console::log_1(&format!("Count changed: {}", *count).into());
///         });
///
///         let increment = callback!([count], |_| {
///             count.set(*count + 1);
///         });
///
///         UI! {
///             <div class="w-[144px]">
///                 <p class="text-2xl font-bold text-center">
///                     "Count: "{*count}
///                 </p>
///                 <div class="text-center">
///                     <button
///                         class="cursor-pointer bg-blue-500 w-[32px] py-1 text-white rounded-md"
///                         onclick={increment}
///                     >"+"</button>
///                 </div>
///             </div>
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! effect {
    ($($t:tt)*) => {
        $crate::laser::effect($crate::callback!($($t)*))
    };
}

pub fn batch(#[cfg_attr(not(client), allow(unused_variables))] f: impl Fn() + 'static) {
    #[cfg(client)]
    {
        let f = Closure::<dyn Fn()>::new(f).into_js_value().unchecked_into();
        preact::batch(f);
    }
}

pub fn untracked(#[cfg_attr(not(client), allow(unused_variables))] f: impl Fn() + 'static) {
    #[cfg(client)]
    {
        let f = Closure::<dyn Fn()>::new(f).into_js_value().unchecked_into();
        preact::untracked(f);
    }
}
