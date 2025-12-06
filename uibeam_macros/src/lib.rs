#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "client")]
mod client;
mod ui;

/// # `UI!` - JSX-style template syntax
///
/// > HTML completions and hovers are available by VSCode extension.\
/// > ( search "_uibeam_" from extension marketplace, or see <https://marketplace.visualstudio.com/items?itemName=ohkami-rs.uibeam> )
///
/// ## Integrations with web frameworks
///
/// Enables `UI` to be returned as a HTML response.
///
/// * [Axum](https://github.com/tokio-rs/axum): by `"axum"` feature
/// * [Actix-web](https://actix.rs): by `"actix-web"` feature
///
/// ## Usage
///
/// ### Serialization
///
/// `UI!` generates a `UI` struct, and `uibeam` provides `shoot(UI) -> Cow<'static, str>`
/// function to serialize the `UI` into HTML string.
///
/// ### Tag Names
///
/// Any HTML tag names are allowed, just the same as JSX.
///
/// ### Attribute Values
///
/// - _string/integer literals_ : Any string/integer literals are allowed. No `{}` is needed.
/// - _interpolations_ : Rust expressions surrounded by `{}` :
///   - `&'static str`, `String`, `Cow<'static, str>` are allowed as string values.
///   - `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize` are allowed as number values.
///   - `bool` is allowed as boolean values.
///
/// ### Text Nodes
///
/// - _string literals_ : Any string literals are allowed. No `{}` is needed.
///   - Raw string literals ( `r#"..."#` ) are **NOT escaped**.
/// - _interpolations_ : Rust expressions surrounded by `{}` :
///   - Any type that implements `std::fmt::Display` is allowed.
///   - Unsafe blocks ( `unsafe { ... }` ) are **NOT escaped**.
///
/// ### Beams
///
/// `<StructName />` or `<StructName></StructName>` are allowed. The structs
/// must implement `uibeam::Beam` trait.
///
/// - `<StructName></StructName>` **requires** the struct to have `children`
///   field. The 0 or more children nodes are passed to `children` as `UI`.
/// - Attributes are interpreted as the struct's fields. Literals are
///   passed as `(it).into()`, and `{any expression}`s are passed directly.
///
/// ```jsx
/// <Struct a="1" b={"2".to_string()} />
///
/// // generates
///
/// Struct {
///     a: ("1").into(),
///     b: "2".to_string(),
/// }
/// ```
///
/// ---
///
/// ```jsx
/// <Struct a="1" b={"2".to_string()}>
///     <p>"hello"</p>
/// </Struct>
///
/// // generates
///
/// Struct {
///     a: ("1").into(),
///     b: "2".to_string(),
///     children: /* a `UI` representing `<p>hello</p>` */
/// }
/// ```
///
/// ---
///
///
/// ## Example
///
/// ```ignore
/// use uibeam::UI;
///
/// fn main() {
///     let user_name = "foo".to_string();
///
///     let style = "
///         color: red; \
///         font-size: 20px; \
///     ";
///     
///     let ui: UI = UI! {
///         <p class="hello" style={style}>
///             "Welcome to the world of UIBeam!"
///             <br>
///             "こんにちは"
///             <a
///                 class="user"
///                 style="color: blue;"
///                 data-user-id="123"
///                 href="https://example-chatapp.com/users/123"
///             >
///                 "@"{user_name}"!"
///             </a>
///         </p>
///     };
///
///     println!("{}", uibeam::shoot(ui));
/// }
/// ```
#[proc_macro]
#[allow(non_snake_case)]
pub fn UI(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ui::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// ## Client Component - WASM islands
///
/// ### overview
///
/// **`#[client]`** makes `Beam` a _*WASM island*_ : initially rendered in server, sent with serialized props, and hydrated with deserialized props in client.
///
/// `Signal`, `computed`, `effect` are available in them.
///
/// ### note
///
/// Currently UIBeam's client component system is built upon [`Preact`](https://preactjs.com).
/// This may be rewritten in pure Rust in the future, but may not because of potential reduction in the final .wasm size.
///
/// ### usage
///
/// working example: [examples/counter](https://github.com/ohkami-rs/uibeam/blob/main/examples/counter)
///
/// 1. Activate `"client"` feature, and add `serde` to your dependencies:
///
///     ```toml
///     [dependencies]
///     uibeam = { version = "0.4" }
///     serde  = { version = "1", features = ["derive"] }
///     ```
///
/// 2. Configure to export all your client components from a specific library crate.
///    (e.g. `lib.rs` entrypoint, or another member crate of a workspace)
///    
///    (There's no problem if including ordinary `Beam`s, not only client ones, in the lib crate.)
///
///    Additionally, specify `crate-type = ["cdylib", "rlib"]` for the crate:
///
///     ```toml
///     [lib]
///     crate-type = ["cdylib", "rlib"]
///     ```
///     
///    For size optimization, following configuration is recommended:
///   
///     ```toml
///     [package.metadata.wasm-pack.release]
///     wasm-opt = ["-Oz"]  # or "-Os"
///     
///     [profile.release]
///     opt-level = 'z'  # or 's'
///     lto = true
///     codegen-unit = 1
///     ```
///
///    See https://rustwasm.github.io/docs/book/game-of-life/code-size.html or other specific documents for more details.
///    
/// 3. Define and use your client components:
///
///     ```rust
///     /* islands/src/lib.rs */
///     
///     use uibeam::{UI, Beam};
///     use uibeam::{client, Signal, callback};
///     use serde::{Serialize};
///     
///     // Client component located at **island boundary**
///     // must be `Serialize`. (see NOTE below)
///     #[derive(Serialize)]
///     pub struct Counter;
///     
///     // `#[client]` makes Beam a Wasm island.
///     // `(island)` means this beam is **island boundary**.
///     #[client(island)]
///     impl Beam for Counter {
///         fn render(self) -> UI {
///             let count = Signal::new(0);
///     
///             // `callback!` - a thin utility for callbacks over signals.
///             let increment = callback!(
///                 // [dependent_signals, ...]
///                 [count],
///                 // closure depending on the signals
///                 |_| count.set(*count + 1)
///             );
///             /* << expanded >>
///     
///             let increment = {
///                 let count = count.clone();
///                 move |_| count.set(*count + 1)
///             };
///             
///             */
///     
///             let decrement = callback!([count], |_| {
///                 count.set(*count - 1)
///             });
///     
///             UI! {
///                 <p>"Count: "{*count}</p>
///                 <button onclick={increment}>"+"</button>
///                 <button onclick={decrement}>"-"</button>
///             }
///         }
///     }
///     ```
///
///     ```rust,ignore
///     /* server/src/main.rs */
///     
///     use islands::Counter;
///     use uibeam::UI;
///     
///     async fn index() -> UI {
///         UI! {
///             <Counter />
///         }
///     }
///     ```
///    
///    **NOTE**:
///    Just `#[client]` components without `(island)`,
///    e.g. one having `children: UI` or `on_something: Box<dyn FnOnce(Event)>` as its props,
///    can **only be used internally in `UI!` of another client component**.
///    Especially note that client components at **island boundary can't have `children`**.
///
/// 4. Compile the lib crate into Wasm by `wasm-pack build` with **`RUSTFLAGS='--cfg hydrate'`** and **`--out-name hydrate --target web`**:
///
///     ```sh
///     # example when naming the lib crate `islands`
///
///     cd islands
///     RUSTFLAGS='--cfg hydrate' wasm-pack build --out-name hydrate --target web
///     ```
///     ```sh
///     # **`--release`** in relase build:
///     
///     RUSTFLAGS='--cfg hydrate' wasm-pack build --out-name hydrate --target web --release
///     ```
///   
///    **NOTE**:
///    All of `RUSTFLAGS='--cfg hydrate'`, `--out-name hydrate` and `--target web` are **required** here.
///
/// 5. Make sure that your server responds with **a complete HTML consist of one <html></html> containing your page contents**.
///    
///    Then, setup your server to serve the output directory (default: `pkg`) at **`/.uibeam`** route:
///  
///     ```rust
///     /* axum example */
///  
///     use axum::Router;
///     use tower_http::services::ServeDir;
///  
///     fn app() -> Router {
///         Router::new()
///             .nest_service(
///                 "/.uibeam",
///                 ServeDir::new("./islands/pkg")
///             )
///             // ...
///     }
///     ```
///
///    (as a result, generated `{crate name}/pkg/hydrate.js` is served at `/.uibeam/hydrate.js` route,
///    which is automatically loaded together with corresponding .wasm file in the hydration step on browser.)
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
#[cfg(feature = "client")]
#[proc_macro_attribute]
pub fn client(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    client::expand(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
