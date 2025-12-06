#!/bin/bash
set -euo pipefail

cargo install -q wasm-pack wasm-bindgen-cli

cd components
cargo test
if [ -d "./pkg" ]; then
    rm -rf ./pkg
fi
RUSTFLAGS='--cfg hydrate' wasm-pack build --out-name hydrate --target web
# check that the `uibeam/src/hydrate.js` is correctly included as a snnipet into `./pkg/hydrate.js`
grep -nER "import \* as [_a-z0-9]+ from '\./snippets/" ./pkg/hydrate.js

cd ..
cargo build

# TODO: `cargo run` with `timeout -s KILL` and
# check serving status (especially JS & Wasm) by headless browser or something similar
