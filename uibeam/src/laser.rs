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
            <div id={id}></div>
            <script type="module">
                "const id = '"{id}"';"
                "const props = JSON.parse('"{props_json}"');"
                "const laser = (await import('lasers.js')).id;"
                "const marker = document.getElementById(id);"
                "marker.parentNode.replaceChild(laser(props), marker);"
            </script>
        }
    }
}
