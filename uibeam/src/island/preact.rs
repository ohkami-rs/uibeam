use ::wasm_bindgen::convert::{FromWasmAbi, IntoWasmAbi};
use ::wasm_bindgen::prelude::*;
use ::js_sys::{Function, Array, Object, Reflect};
use ::web_sys::Node;

mod preact {
    use super::*;
    
    #[wasm_bindgen(module = "preact")]
    unsafe extern "C" {
        #[wasm_bindgen(js_name = "hydrate")]
        pub(super) fn hydrate(vdom: JsValue, container: Node);

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
        pub(super) fn fragment() -> JsValue;
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



pub fn signal<T: JsCast>(value: T) -> (
    impl (Fn() -> T) + Copy + 'static,
    impl (Fn(T)) + Copy + 'static
) {
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

pub fn computed<T: JsCast>(
    getter: impl (Fn() -> T) + 'static
) -> impl (Fn() -> T) + Copy + 'static {
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

pub fn effect(
    f: impl Fn() + 'static
) {
    let f = Closure::<dyn Fn()>::new(f)
        .into_js_value()
        .unchecked_into();

    preact::effect(f);
}

pub fn batch(
    f: impl Fn() + 'static
) {
    let f = Closure::<dyn Fn()>::new(f)
        .into_js_value()
        .unchecked_into();

    preact::batch(f);
}

pub fn untracked(
    f: impl Fn() + 'static
) {
    let f = Closure::<dyn Fn()>::new(f)
        .into_js_value()
        .unchecked_into();

    preact::untracked(f);
}
