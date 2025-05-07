mod ui;

#[proc_macro]
#[allow(non_snake_case)]
/// # `UI!` - JSX-style template syntax
/// 
/// <br>
/// 
/// ---
/// 
/// HTML completions and hovers are available by VSCode extension.\
/// ( search "_uibeam_" from extension marketplace, or see https://marketplace.visualstudio.com/items?itemName=ohkami-rs.uibeam )
/// 
/// ---
/// 
/// <br>
/// 
/// ## Usage
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
/// - _interpolations_ : Rust expressions surrounded by `{}` :
///   - Any type that implements `std::fmt::Display` is allowed.
/// 
/// <br>
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
