#![cfg(feature = "laser")]

#[doc(hidden)]
pub use ::wasm_bindgen;

use crate::{UI, Beam};
use serde::Serialize;

pub trait Laser: Serialize {
    const ID: &'static str;

    fn boot(self) -> UI;
}

impl<L: Laser> Beam for L {
    fn render(self) -> UI {
        let props_json = serde_json::to_string(&self).unwrap();

        UI! {
            <div data-laser-id={L::ID}></div>
            <script type="module">
                "const ID = '"{L::ID}"';"
                "const props = JSON.parse('"{props_json}"');"
                "const laser = (await import('lasers.js')).ID;"
                "const marker = document.getElementById(ID);"
                "marker.parentNode.replaceChild(laser(props), marker);"
            </script>
        }
    }
}
