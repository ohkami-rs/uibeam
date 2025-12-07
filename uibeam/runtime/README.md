### `vendor/*.mjs`

Vendored *specific version* of

- `preact`
- `preact/hooks`
- `@preact/signals`
- `@preact/signals-core`

from [esm.sh](https://esm.sh/), with a few manual patches, under MIT Liscense.

### `runtime.js`

Loaded by `/.uibeam/hydrate.js` and works as the runtime entrypoint that
starts hydration for Client Beams on browser.

### `bundle.sh`

A script that bundles (and minifies) the whole five js files,
internally using *specific version* of [`esbuild`](https://github.com/evanw/esbuild).

### `../runtime.mjs`

The output of `bundle.sh`.
