mod ui;

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
/// - _string literals_ : Any string literals are allowed. No `{}` is needed.
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
/// - Attributes are interpreted as the struct's fields. The values are
///   passed to each field with `.into()`.
/// 
/// ```jsx
/// <Struct a="1" b="2" />
/// 
/// // generates
/// 
/// Struct {
///     a: "1".into(),
///     b: "2".into(),
/// }
/// ```
/// 
/// ---
/// 
/// ```jsx
/// <Struct a="1" b="2">
///     <p>"hello"</p>
/// </Struct>
/// 
/// // generates
/// 
/// Struct {
///     a: "1".into(),
///     b: "2".into(),
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
mod laser;

#[cfg(feature = "laser")]
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Laser(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    laser::expand(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_attribute]
pub fn consume(_: proc_macro::TokenStream, _: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
