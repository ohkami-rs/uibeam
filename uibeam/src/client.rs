#![cfg(feature = "client")]

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

#[cfg(not(hydrate))]
pub(crate) mod server_gc {
    use std::{cell::RefCell, any::Any};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    
    thread_local! {
        static GC: RefCell<Vec<Box<dyn Any>>> = RefCell::new(Vec::new());
    }
    
    pub(super) struct Pointer<T>(std::ptr::NonNull<T>);
    impl<T> Clone for Pointer<T> {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T> Copy for Pointer<T> {}
    impl<T> Pointer<T> {
        /// SAFETY: When calling this method, you have to ensure that the pointer is convertible to a reference.
        pub(super) unsafe fn as_ref(&self) -> &T {
            // SAFETY: by caller's guarantee
            unsafe { self.0.as_ref() }
        }
    }
    
    pub(super) fn alloc<T: 'static>(value: T) -> Pointer<T> {
        let raw = Box::into_raw(Box::new(value));
        // SAFETY: `raw` is valid pointer allocated from Box.
        unsafe {            
            GC.with_borrow_mut(|vec| vec.push(Box::from_raw(raw)));
            Pointer(std::ptr::NonNull::new_unchecked(raw))
        }
    }
    
     pub(crate) fn run_in_gc_context<R>(f: impl FnOnce() -> R) -> R {
         let result = catch_unwind(AssertUnwindSafe(|| f())).unwrap_or_else(|e| {
             if let Some(s) = e.downcast_ref::<String>() {
                 panic!("{s}");
             } else if let Some(s) = e.downcast_ref::<&str>() {
                 panic!("{s}");
             } else {
                 panic!("uibeam: panic within shoot context");
             }
         });
         GC.with_borrow_mut(|vec| vec.clear());
         result
     }
}

#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub struct Signal<T> {
    #[cfg(hydrate)]
    alien_signal: ::alien_signals::Signal<T>,
    /// A dummy signal value just for template rendering,
    /// not handling any updates.
    #[cfg(not(hydrate))]
    value: server_gc::Pointer<T>,
}
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            #[cfg(hydrate)]
            alien_signal: self.alien_signal.clone(),
            #[cfg(not(hydrate))]
            value: self.value.clone(),
        }
    }
}
impl<T> Copy for Signal<T> {}
impl<T: Clone + 'static> Signal<T> {
    pub fn new(init: T) -> Self {
        Self {
            #[cfg(hydrate)]
            alien_signal: ::alien_signals::Signal::new(init),
            #[cfg(not(hydrate))]
            value: server_gc::alloc(init),
        }
    }
    pub fn new_with_eq_fn(
        init: T,
        #[allow(unused)]
        eq_fn: impl Fn(&T, &T) -> bool + 'static,
    ) -> Self {
        Self {
            #[cfg(hydrate)]
            alien_signal: ::alien_signals::Signal::new_with_eq_fn(init, eq_fn),
            #[cfg(not(hydrate))]
            value: server_gc::alloc(init),
        }
    }
    
    pub fn get(&self) -> T {
        #[cfg(hydrate)]
        {
            self.alien_signal.get()
        }
        #[cfg(not(hydrate))]
        {
            // SAFETY: value is never deallocated during the lifetime of the Signal.
            (unsafe {self.value.as_ref()}).clone()
        }
    }
    
    pub fn set(&self, value: T) {
        #[cfg(hydrate)]
        {
            self.alien_signal.set(value);
        }
        #[cfg(not(hydrate))]
        {
            // no-op
            let _ = value;
        }
    }    
    /// set with current value
    pub fn set_with(&self, f: impl FnOnce(&T) -> T) {
        #[cfg(hydrate)]
        {
            self.alien_signal.set_with(f);
        }
        #[cfg(not(hydrate))]
        {
            // no-op
            let _ = f;
        }
    }
    /// set with mutating the value
    pub fn set_with_mut(&self, f: impl FnOnce(&mut T)) {
        #[cfg(hydrate)]
        {
            self.alien_signal.update(f);
        }
        #[cfg(not(hydrate))]
        {
            // no-op
            let _ = f;
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
pub struct Computed<T> {
    #[cfg(hydrate)]
    alien_computed: ::alien_signals::Computed<T>,
    /// A dummy computed value just for template rendering,
    /// not handling any updates.
    #[cfg(not(hydrate))]
    value: server_gc::Pointer<T>,
}
impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            #[cfg(hydrate)]
            alien_computed: self.alien_computed.clone(),
            #[cfg(not(hydrate))]
            value: self.value.clone(),
        }
    }
}
impl<T> Copy for Computed<T> {}
impl<T: Clone + 'static> Computed<T> {
    pub fn new(f: impl Fn() -> T + 'static) -> Self {
        Self {
            #[cfg(hydrate)]
            alien_computed: ::alien_signals::Computed::new(f),
            #[cfg(not(hydrate))]
            value: server_gc::alloc(f()),
        }
    }
    pub fn new_with_eq_fn(
        f: impl Fn() -> T + 'static,
        #[allow(unused)]
        eq_fn: impl Fn(&T, &T) -> bool + 'static,
    ) -> Self {
        Self {
            #[cfg(hydrate)]
            alien_computed: ::alien_signals::Computed::new_with_eq_fn(f, eq_fn),
            #[cfg(not(hydrate))]
            value: server_gc::alloc(f()),
        }
    }
    
    pub fn get(&self) -> T {
        #[cfg(hydrate)]
        {
            self.alien_computed.get()
        }
        #[cfg(not(hydrate))]
        {
            // SAFETY: value is never deallocated during the lifetime of the Computed.
            (unsafe {self.value.as_ref()}).clone()
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
#[derive(Clone, Copy)]
pub struct Effect {
    #[cfg(hydrate)]
    alien_effect: ::alien_signals::Effect,
}
impl Effect {
    pub fn new(f: impl Fn() + 'static) -> Self {
        #[cfg(hydrate)]
        {            
            Self { alien_effect: ::alien_signals::Effect::new(f) }
        }
        #[cfg(not(hydrate))]
        {
            f();
            Self {}
        }
    }
    
    pub fn dispose(self) {
        #[cfg(hydrate)]
        {
            self.alien_effect.dispose();
        }
        #[cfg(not(hydrate))]
        {
            // no-op
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
#[derive(Clone, Copy)]
pub struct EffectScope {
    #[cfg(hydrate)]
    alien_effect_scope: ::alien_signals::EffectScope,
}
impl EffectScope {
    pub fn new(f: impl FnOnce() + 'static) -> Self {
        #[cfg(hydrate)]
        {            
            Self { alien_effect_scope: ::alien_signals::EffectScope::new(f) }
        }
        #[cfg(not(hydrate))]
        {
            f();
            Self {}
        }
    }
    
    pub fn dispose(self) {
        #[cfg(hydrate)]
        {
            self.alien_effect_scope.dispose();
        }
        #[cfg(not(hydrate))]
        {
            // no-op
        }
    }
}
