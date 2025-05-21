mod preact;

pub use preact::*;

#[doc(hidden)]
pub use ::wasm_bindgen;

pub trait Laser {
    fn render(self) -> crate::UI;
}
