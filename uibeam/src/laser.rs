#![cfg(feature = "laser")]

#[doc(hidden)]
pub use ::wasm_bindgen;

use crate::{UI, Beam};

pub trait Laser: ::serde::Serialize {
    fn boot(self) -> UI;
}

impl<L: Laser> Beam for L {
    fn render(self) -> UI {
        let id = std::any::type_name::<L>();

        let props_json = ::serde_json::to_string(&self).unwrap();

        UI! {
            <div data-laser-id={id}></div>
            <script type="module">
                "const ID = '"{id}"';"
                "const props = JSON.parse('"{props_json}"');"
                "const laser = (await import('lasers.js')).ID;"
                "const marker = document.getElementById(ID);"
                "marker.parentNode.replaceChild(laser(props), marker);"
            </script>
        }
    }
}
