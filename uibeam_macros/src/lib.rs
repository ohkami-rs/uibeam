mod ui;
#[cfg(feature = "laser")]
mod laser;

#[proc_macro]
#[allow(non_snake_case)]
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
pub fn UI(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ui::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[cfg(feature = "laser")]
#[proc_macro_attribute]
#[allow(non_snake_case)]
/// # `Laser` - Client component by WASM island
/// 
/// ## architecture
/// 
/// `Laser` trait provides a way to build client components in WASM. They works as _*islands*_ : initially rendered in server, sent with serialized props, and hydrated with deserialized props in client.
/// 
/// `Signal`, `computed`, `effect` are available in `Laser`s.
/// 
/// At current version (v0.3), `Laser` system is built up on [Preact](https://preactjs.com).
/// 
/// This is experimental design choice and maybe fully/partially replaced into some Rust implementaion in future. <i>(But this may be kind of better choice, for example, at avoiding huge size of WASM output.)</i>
/// 
/// ## Usage
/// 
/// 1. Activate `"laser"` feature, and add `serde`:
/// 
///     ```toml
///     [dependencies]
///     uibeam = { version = "0.3.0", features = ["laser"] }
///     serde  = { version = "1", features = ["derive"] }
///     ```
/// 
/// 2. Create an UIBeam-specific library crate (e.g. `lasers`) as a workspace member, and have all `Laser`s in that crate. (of cource, no problem if including all `Beam`s not only `Laser`s. Then the name of this crate should be `components` or something.)
/// 
///    Make sure to specify `crate-type = ["cdylib", "rlib"]`:
/// 
///     ```toml
///     [lib]
///     crate-type = ["cdylib", "rlib"]
///     ```
///    
/// 3. Build your `Laser`s:
/// 
///     ```rust
///     use uibeam::{UI, Laser, Signal, callback};
///     use serde::{Serialize, Deserialize};
///     
///     #[Laser]
///     #[derive(Serialize, Deserialize)]
///     struct Counter;
///     
///     impl Laser for Counter {
///         fn render(self) -> UI {
///             let count = Signal::new(0);
///     
///             // callback utility
///             let increment = callback!(
///                 // dependent signals
///                 [count],
///                 // |args, ...| expression
///                 |_| count.set(*count + 1)
///             );
///     
///             /* expanded:
///     
///             let increment = {
///                 let count = count.clone();
///                 move |_| count.set(*count + 1)
///             };
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
///     `#[Laser(local)]` ebables to build _**local Lasers**_:
///     
///     - not require `Serialize` `Deserialize` and can have unserializable items in its fields such as `fn(web_sys::Event)`.
///     - only available as a `UI` element of a non-local `Laser` or other local `Laser`.\
///       otherwise: **not hydrated**. currently this is silent behavior. (maybe rejected by compile-time check in future version)
/// 
/// 4. Compile to WASM by `wasm-pack build` with **`--target web --out-name lasers`**:
/// 
///     ```sh
///     # example when naming the crate `components`
/// 
///     cd components
///     wasm-pack build --target web --out-name lasers
/// 
///     # or
/// 
///     wasm-pack build components --target web --out-name lasers
///     ```
/// 
///    and set up to serve the output directly (default: `pkg`) at **`/.uibeam`**:
///  
///     ```rust
///     // axum example
///  
///     use axum::Router;
///     use tower_http::services::ServeDir;
///  
///     fn app() -> Router {
///         Router::new()
///             .nest_service(
///                 "/.uibeam",
///                 ServeDir::new("./components/pkg")
///             )
///             // ...
///     }
///     ```
/// 
///    As a result, `components/pkg/lasers.js` is served at `/.uibeam/lasers.js` and automatically loaded together with WASM by a Laser in the first hydration.
pub fn Laser(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    laser::expand(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn consume(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
