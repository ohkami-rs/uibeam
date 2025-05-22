#![cfg(feature = "laser")]

use ::wasm_bindgen::prelude::*;

#[doc(hidden)]
pub use {::wasm_bindgen, ::web_sys};

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

impl<L: Laser + ::serde::Serialize> ::uibeam::Beam for L {
    fn render(self) -> ::uibeam::UI {
        #[cfg(target_arch = "wasm32")] {
            unreachable!();
        }

        #[cfg(not(target_arch = "wasm32"))] {
            let name = format!("__uibeam_laser_{}__", type_ident::<L>());

            let props: String = ::uibeam::laser::serialize_props(&self);

            let template: ::std::borrow::Cow<'static, str> = ::uibeam::shoot(<Self as Laser>::render(self));

            ::uibeam::UI! {
                <div
                    data-uibeam-laser={name.clone()}
                >
                    unsafe {template}

                    <script type="module">
unsafe {format!("
const name = '{name}';
const props = JSON.parse('{props}');
")}
r#"
if (window.__uibeam_initlock__) {
    while (!window.__uibeam_lasers__) await new Promise(resolve => setTimeout(resolve, 100));
} else {
    window.__uibeam_initlock__ = true;
    const { default: init, ..lasers } = await import('./pkg/lasers.js');
    await init();
    window.__uibeam_lasers__ = lasers;
}
(window.__uibeam_lasers__[name])(
    props,
    document.querySelector(`[data-uibeam-laser=${name}]`)
);
"#
                    </script>
                </div>
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod preact {
    use super::*;
    
    #[wasm_bindgen(module = "https://esm.sh/preact")]
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

    #[wasm_bindgen(module = "https://esm.sh/@preact/signals?external=preact")]
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
    ::js_sys::{Function, Array, Object, Reflect},
    ::wasm_bindgen::convert::{FromWasmAbi, IntoWasmAbi, TryFromJsValue},
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
        L: Laser + TryFromJsValue<Error = JsValue>,
    {
        let component_function: Function = Closure::<dyn Fn(JsValue)->JsValue>::new(|props| {
            let props = <L as TryFromJsValue>::try_from_js_value(props).unwrap_throw();
            <L as Laser>::render(props).into_vdom().0
        }).into_js_value().unchecked_into();

        let ident = JsValue::from(type_ident::<L>());
        Reflect::set(&component_function, &"name".into(), &ident).ok();
        Reflect::set(&component_function, &"displayName".into(), &ident).ok();

        NodeType(component_function.unchecked_into())
    }
}

#[cfg(target_arch = "wasm32")]
impl VNode {
    pub fn new(
        r#type: NodeType,
        props: Vec<(&'static str, JsValue)>,
        children: Vec<VNode>,
    ) -> VNode {
        let props_entries = {
            let entries = props.into_iter().map(|(k, v)| {
                let entry = [k.into(), v].into_iter().collect::<Array>();
                let entry: JsValue = entry.unchecked_into();
                entry
            }).collect::<Array>();
            let entries: JsValue = entries.unchecked_into();
            entries
        };

        VNode(preact::create_element(
            r#type.0,
            Object::from_entries(&props_entries).unwrap_throw(),
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

pub fn signal<T: JsCast + Clone + 'static>(value: T) -> (
    impl (Fn() -> T) + Clone + 'static,
    impl (Fn(T)) + Clone + 'static
) {
    #[cfg(not(target_arch = "wasm32"))] {// for template rendering
        (
            move || value.clone(),
            |_value: T| {}
        )
    }
    #[cfg(target_arch = "wasm32")] {
        let signal = preact::signal(value.unchecked_into());
        let signal = Object::into_abi(signal);

        let get = move || {
            let signal = unsafe {Object::from_abi(signal)};
            Reflect::get(&signal, &"value".into())
                .unwrap_throw()
                .unchecked_into()
        };

        let set = move |value: T| {
            let signal = unsafe {Object::from_abi(signal)};
            Reflect::set(&signal, &"value".into(), &value.unchecked_into())
                .unwrap_throw();
        };

        (get, set)
    }
}

pub fn computed<T: JsCast + Clone + 'static>(
    getter: impl (Fn() -> T) + Clone + 'static
) -> impl (Fn() -> T) + Clone + 'static {
    #[cfg(not(target_arch = "wasm32"))] {// for template rendering
        getter
    }
    #[cfg(target_arch = "wasm32")] {
        let getter = Closure::<dyn Fn()->JsValue>::new(move || getter().unchecked_into())
            .into_js_value()
            .unchecked_into();

        let computed = preact::computed(getter);
        let computed = Object::into_abi(computed);

        move || {
            let computed = unsafe {Object::from_abi(computed)};
            Reflect::get(&computed, &"value".into())
                .unwrap_throw()
                .unchecked_into()
        }
    }
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
