#![cfg(feature = "laser")]

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
    ::wasm_bindgen::convert::{FromWasmAbi, IntoWasmAbi, TryFromJsValue},
};

#[cfg(target_arch = "wasm32")]
fn type_ident<T>() -> &'static str {
    let type_name = std::any::type_name::<T>();
    let type_path = if type_name.ends_with('>') {
        /* `type_name` has generics like `playground::handler<alloc::string::String>` */
        /* ref: <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=e02e32853dddf5385769d1718c481814> */
        let (type_path, _/*generics*/) = type_name
            .rsplit_once('<')
            .expect("unexpectedly independent `>` in std::any::type_name");
        type_path
    } else {
        type_name
    };
    let (_/*path from crate root*/, type_ident) = type_path
        .rsplit_once("::")
        .expect("unexpected format of std::any::type_name");
    type_ident
}

#[cfg(target_arch = "wasm32")]
pub fn hydrate(
    vdom: VNode,
    // component: impl Laser + serde::Serialize,
    container: ::web_sys::Node,
) {
    ::web_sys::console::log_2(
        &"Hydrating VNode: ".into(),
        &vdom.0,
    );

    preact::hydrate(vdom.0, container);
}

#[cfg(target_arch = "wasm32")]
pub struct VNode(JsValue);

#[cfg(target_arch = "wasm32")]
pub struct NodeType(JsValue);

#[cfg(target_arch = "wasm32")]
impl NodeType {
    pub fn tag(tag: &'static str) -> NodeType {
        ::web_sys::console::log_1(&format!("Creating NodeType for tag: {}", tag).into());

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

        let ident = JsValue::from(type_ident::<L>());
        Reflect::set(&component_function, &"name".into(), &ident).ok();
        Reflect::set(&component_function, &"displayName".into(), &ident).ok();

        ::web_sys::console::log_2(
            &"Creating NodeType for component: ".into(),
            component_function.unchecked_ref(),
        );

        NodeType(component_function.unchecked_into())
    }
}

#[cfg(target_arch = "wasm32")]
impl VNode {
    pub fn new(
        r#type: NodeType,
        props: Object,//Vec<(&'static str, JsValue)>,
        children: Vec<VNode>,
    ) -> VNode {
        ::web_sys::console::log_2(
            &"Creating VNode with type: ".into(),
            &r#type.0,
        );

        VNode(preact::create_element(
            r#type.0,
            props,//Object::from_entries(&props_entries).unwrap_throw(),
            children.into_iter().map(|vdom| vdom.0).collect::<Array>(),
        ))
    }

    pub fn fragment(
        children: Vec<VNode>,
    ) -> VNode {
        ::web_sys::console::log_1(&"Creating VNode fragment".into());

        let props = Object::new();
        Reflect::set(
            &props,
            &"children".into(),
            &children.into_iter().map(|vdom| vdom.0).collect::<Array>(),
        ).ok();
        VNode(preact::fragment(props))
    }

    pub fn text(text: impl Into<std::borrow::Cow<'static, str>>) -> VNode {
        ::web_sys::console::log_1(&format!("Creating VNode text").into());

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

pub fn signal<T: serde::Serialize + for<'de>serde::Deserialize<'de> + 'static>(
    value: T
) -> Signal<T> {
    Signal::new(value)
}

pub fn computed<T: serde::Serialize + for<'de>serde::Deserialize<'de> + 'static>(
    getter: impl (Fn() -> T) + 'static
) -> Computed<T> {
    Computed::new(getter)
}

#[macro_export]
macro_rules! computed {
    (|| $result:expr) => {
        $crate::laser::computed(|| $result)
    };
    (move || $result:expr) => {
        $crate::laser::computed(move || $result)
    };
    ($dep_signal:ident => $result:expr) => {
        $crate::laser::computed({
            let $dep_signal = $dep_signal.clone();
            move || $result
        })
    };
    (($($dep_signal:ident),*) => $result:expr) => {
        $crate::laser::computed({
            $(let $dep_signal = $dep_signal.clone();)+
            move || $result
        })
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
    (|| $result:expr) => {
        $crate::laser::effect(|| $result)
    };
    (move || $result:expr) => {
        $crate::laser::effect(move || $result)
    };
    ($dep_signal:ident => $result:expr) => {
        $crate::laser::effect({
            let $dep_signal = $dep_signal.clone();
            move || $result
        })
    };
    (($($dep_signal:ident),*) => $result:expr) => {
        $crate::laser::effect({
            $(let $dep_signal = $dep_signal.clone();)+
            move || $result
        })
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
