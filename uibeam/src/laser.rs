#![cfg(feature = "laser")]

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
                r#"
const lasers = await import('lasers.js');

const laser = lasers.ID;
if (!laser) {
    console.error(`[UIBeam] Laser with ID '${ID}' not found.`);
    return;
}

const marker = document.getElementById(ID);
if (!marker) {
    console.error(`[UIBeam] Laser marker with ID '${ID}' not found.`);
    return;
}

marker.parentNode.replaceChild(laser(props), marker);
                "#
            </script>
        }
    }
}
