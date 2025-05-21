mod preact;

pub use preact::*;

#[doc(hidden)]
pub use {::wasm_bindgen, ::web_sys};

pub trait Laser {
    fn render(self) -> crate::UI;
}
