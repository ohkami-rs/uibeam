#![cfg_attr(docsrs, feature(doc_cfg))]

/* execute doc tests for sample codes in README */
#![cfg_attr(all(doc, not(docsrs)), doc = include_str!("../../README.md"))]

//! <div align="center">
//!     <h1>
//!         UIBeam
//!     </h1>
//!     <p>
//!         A lightweight, JSX-style HTML template engine for Rust
//!     </p>
//! </div>
//! 
//! - `UI!` : JSX-style template syntax with compile-time checks
//! - `Beam` : Component system
//! - Simple : Simply organized API and codebase, with zero external dependencies
//! - Efficient : Emitting efficient codes, avoiding redundant memory allocations as smartly as possible
//! - Better UX : HTML completions and hovers in `UI!` by VSCode extension ( search by "_uibeam_" from extension marketplace )
//! 
//! ![](https://github.com/ohkami-rs/uibeam/raw/HEAD/support/vscode/assets/completion.png)

/* for `UI!` use in this crate itself */
extern crate self as uibeam;

#[cfg(feature = "__integration__")]
mod integration;

use std::borrow::Cow;

pub use uibeam_html::escape;
pub use uibeam_macros::UI;

/// # `UI` - UIBeam's template representation
/// 
/// Generated by [ `UI!` ](macro@UI) macro, and serialized into `Cow<'static, str>` by [`shoot`] function.\
/// See `UI!` for more details.
pub struct UI(Cow<'static, str>);

/// # `Beam` - UIBeam's component system
/// 
/// <br>
/// 
/// ## Usage
/// 
/// When `StructName` implements `Beam`, `<StructName />` or `<StructName></StructName>`
///  are available in `UI!`.
/// 
/// - `<StructName></StructName>` **requires** the struct to have `children`
///   field. The 0 or more children nodes are passed to `children` as `UI`.
/// - Attributes are interpreted as the struct's fields. The values are
///   passed to each field with `.into()`.
/// 
/// <br>
/// 
/// ---
/// 
/// <br>
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
/// <br>
/// 
/// ---
/// 
/// <br>
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
/// <br>
/// 
/// ---
/// 
/// <br>
/// 
/// ## Example
/// 
/// ```
/// use uibeam::{UI, Beam};
/// 
/// struct MyComponent {
///     name: String,
///     age: u8,
/// }
/// 
/// impl Beam for MyComponent {
///     fn render(self) -> UI {
///         UI! {
///             <div>
///                 <p>"My name is "{self.name}</p>
///                 <p>"My age is "{self.age}</p>
///             </div>
///         }
///     }
/// }
/// 
/// fn main() {
///     let ui = UI! {
///         <h1>"Hello, Beam!"</h1>
///         <MyComponent name={"Alice"} age={30} />
///     };
///     let html = uibeam::shoot(ui);
///     println!("{}", html);
/// }
/// ```
/// 
pub trait Beam {
    fn render(self) -> UI;
}

#[inline(always)]
pub fn shoot(ui: UI) -> Cow<'static, str> {
    ui.0
}

impl FromIterator<UI> for UI {
    #[inline]
    fn from_iter<T: IntoIterator<Item = UI>>(iter: T) -> Self {
        let mut result = String::new();
        for item in iter {
            result.push_str(&item.0);
        }
        UI(Cow::Owned(result))
    }
}

impl UI {
    pub const EMPTY: UI = UI(Cow::Borrowed(""));

    #[inline(always)]
    pub fn concat<const N: usize>(uis: [UI; N]) -> Self {
        match uis.len() {
            0 => UI::EMPTY,
            1 => unsafe {
                // SAFETY:
                // * original `uis` is moved to this function
                //   and never used again by anyone
                // * `ManuallyDrop` prevents double free
                //   and returned `Self`'s destructor will free this memory
                std::ptr::read(
                    // SAFETY:
                    // * Here `uis` is `[UI; 1]`
                    // * `[T; 1]` has the same layout as `T`
                    // * `ManuallyDrop<T>` has the same layout as `T`
                    &*std::mem::ManuallyDrop::new(uis)
                        as *const [UI]
                        as *const [UI; 1]
                        as *const UI
                )
            }
            _ => {
                let mut buf = String::with_capacity(uis.iter().map(|ui| ui.0.len()).sum());
                for ui in uis {
                    buf.push_str(&ui.0);
                }
                UI(Cow::Owned(buf))
            }
        }
    }
}

#[doc(hidden)]
pub enum Interpolator {
    /// interpolation of a HTML attribute value:
    /// - `class={foo}`
    /// - `checked={true}`
    /// - `width={100}`
    Attribute(AttributeValue),
    /// interpolation of HTML elements or nodes within a parent element:
    /// - `<div>{children}</div>`
    /// - `<div>{iter.map(|i| UI! { ... })}</div>`
    /// - `<div>{condition.then(|| UI! { ... })}</div>`
    /// - `<p>My name is {me.name}</p>` (in text node)
    Children(UI),
}

#[doc(hidden)]
pub enum AttributeValue {
    Text(Cow<'static, str>),
    Integer(i64),
    Boolean(bool),
}
const _: () = {
    impl From<bool> for AttributeValue {
        #[inline(always)]
        fn from(value: bool) -> Self {
            AttributeValue::Boolean(value)
        }
    }

    impl From<&'static str> for AttributeValue {
        fn from(value: &'static str) -> Self {
            AttributeValue::Text(value.into())
        }
    }
    impl From<String> for AttributeValue {
        #[inline(always)]
        fn from(value: String) -> Self {
            AttributeValue::Text(value.into())
        }
    }
    impl From<Cow<'static, str>> for AttributeValue {
        fn from(value: Cow<'static, str>) -> Self {
            AttributeValue::Text(value)
        }
    }

    impl From<i8> for AttributeValue {
        fn from(it: i8) -> Self {
            AttributeValue::Integer(it.into())
        }
    }
    impl From<i16> for AttributeValue {
        fn from(it: i16) -> Self {
            AttributeValue::Integer(it.into())
        }
    }
    impl From<i32> for AttributeValue {
        #[inline(always)]
        fn from(it: i32) -> Self {
            AttributeValue::Integer(it.into())
        }
    }
    impl From<i64> for AttributeValue {
        fn from(it: i64) -> Self {
            AttributeValue::Integer(it)
        }
    }
    impl From<isize> for AttributeValue {
        fn from(it: isize) -> Self {
            AttributeValue::Integer(it.try_into().expect(&too_large_error_message(it)))
        }
    }
    impl From<u8> for AttributeValue {
        fn from(it: u8) -> Self {
            AttributeValue::Integer(it.into())
        }
    }
    impl From<u16> for AttributeValue {
        fn from(it: u16) -> Self {
            AttributeValue::Integer(it.into())
        }
    }
    impl From<u32> for AttributeValue {
        #[inline(always)]
        fn from(it: u32) -> Self {
            AttributeValue::Integer(it.into())
        }
    }
    impl From<u64> for AttributeValue {
        fn from(it: u64) -> Self {
            AttributeValue::Integer(it.try_into().expect(&too_large_error_message(it)))
        }
    }
    impl From<usize> for AttributeValue {
        fn from(it: usize) -> Self {
            AttributeValue::Integer(it.try_into().expect(&too_large_error_message(it)))
        }
    }
    #[cold]
    #[inline(never)]
    fn too_large_error_message(int: impl std::fmt::Display) -> String {
        format!("can't use `{int}` as attribute value: too largem")
    }
};

#[doc(hidden)]
pub trait IntoChildren<T, const ESCAPE: bool = true> {
    fn into_children(self) -> UI;
}
const _: () = {
    impl<const ESCAPE: bool> IntoChildren<UI, ESCAPE> for UI {
        fn into_children(self) -> UI {
            self
        }
    }

    // note that `Option<UI>` implements `IntoChildren` because `Option` is `IntoIterator`
    impl<const ESCAPE: bool, I> IntoChildren<(I,), ESCAPE> for I
    where
        I: IntoIterator<Item = UI>,
    {
        #[inline(always)]
        fn into_children(self) -> UI {
            UI::from_iter(self)
        }
    }

    impl<const ESCAPE: bool, D: std::fmt::Display> IntoChildren<&dyn std::fmt::Display, ESCAPE> for D {
        fn into_children(self) -> UI {
            let s = self.to_string();
            if ESCAPE {
                match escape(&s) {
                    Cow::Owned(escaped) => {
                        UI(Cow::Owned(escaped))
                    }
                    Cow::Borrowed(_) => {
                        // this means `s` is already escaped, so we can avoid allocation,
                        // just using `s` directly
                        UI(Cow::Owned(s))
                    }
                }
            } else {
                UI(Cow::Owned(s))
            }
        }
    }
};

#[doc(hidden)]
impl UI {
    /// tends to be used by the `UI!` macro internally.
    /// 
    /// ## SAFETY
    /// 
    /// 1. `template_pieces` must have 0 = N or exactly `N + 1` pieces.
    /// 2. `template_pieces` must be concatenated into
    ///    a valid HTML string with any `interpolators` in place.
    /// 3. Each piece in `template_pieces` must be already HTML-escaped.
    ///    (intended to be escaped in `UI!` macro internally /
    ///     `new_unchecked` itself does not check or escape)
    pub unsafe fn new_unchecked<const N: usize>(
        template_pieces: &'static [&'static str],
        interpolators: [Interpolator; N],
    ) -> Self {
        #[cfg(debug_assertions)] {
            let len = template_pieces.len();
            assert!(
                (len == 0 && N == 0) || len == N + 1,
                "invalid template_pieces.len(): {len} where N = {N}: template_pieces must have 0 = N or exactly N + 1 pieces"
            );
        }

        match template_pieces.len() {
            0 => UI::EMPTY,
            1 => UI(Cow::Borrowed(template_pieces[0])),
            _ => {
                let mut buf = String::with_capacity({
                    let mut size = 0;
                    for piece in template_pieces {
                        size += piece.len();
                    }
                    for expression in &interpolators {
                        size += match expression {
                            Interpolator::Children(children) => children.0.len(),
                            Interpolator::Attribute(value) => match value {
                                AttributeValue::Text(text) => {
                                    1/* " */ + text.len() + 1/* " */
                                }
                                AttributeValue::Integer(_) => {
                                    1/* " */ + 4/* max-class length of typically used integer attribute values */ + 1/* " */
                                }
                                AttributeValue::Boolean(_) => {
                                    0/* not push any tokens */
                                }
                            }
                        }
                    }
                    size
                });

                for i in 0..N {
                    buf.push_str(template_pieces[i]);
                    match &interpolators[i] {
                        Interpolator::Children(children) => {
                            buf.push_str(&children.0);
                        }
                        Interpolator::Attribute(value) => {
                            #[cfg(debug_assertions)] {
                                // expect like
                                // 
                                // ```in UI!{}
                                // <div class={}
                                //            |
                                //            /-- this `value` is here
                                // ```
                                assert!(buf.ends_with('='));
                            }
                            match value {
                                AttributeValue::Text(text) => {
                                    buf.push('"');
                                    buf.push_str(&escape(text));
                                    buf.push('"');
                                }
                                AttributeValue::Integer(int) => {
                                    // here we don't need to escape
                                    buf.push('"');
                                    buf.push_str(&int.to_string());
                                    buf.push('"');
                                }
                                AttributeValue::Boolean(boolean) => {
                                    // if `boolean` is `true`, we'll just leave the attribute name :
                                    // 
                                    // ```in UI!{}
                                    // <input type="checkbox" checked={true}
                                    // 
                                    // // to
                                    // 
                                    // <input type="checkbox" checked
                                    // ```
                                    // 
                                    // if `boolean` is `false`, we'll remove up to the attribute name :
                                    //
                                    // ```in UI!{}
                                    // <input type="checkbox" checked={false}
                                    //
                                    // // to
                                    //
                                    // <input type="checkbox"
                                    // ```
                                    //
                                    // this can be done by removing after the last whitespace of current `buf`
                                    // (because the SAFETY contract encusres `buf` is a part of a valid HTML string
                                    // and then at least one whitespace exists before an attribute name)
                                    let Some('=') = buf.pop() else {unreachable!()};
                                    if !*boolean {
                                        let Some(sp) = buf.rfind(is_ascii_whitespace) else {unreachable!()};
                                        buf.truncate(sp);
                                    }
                                }
                            }
                        }
                    }
                }
                buf.push_str(template_pieces[N]);
                UI(Cow::Owned(buf))
            }
        }
    }
}

#[inline(always)]
const fn is_ascii_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0C' | '\r')
}

#[cfg(test)]
mod test {
    use super::*;

    /* compiles */
    fn __assert_impls__() {
        fn is_children<X, C: IntoChildren<X>>(_: C) {}
        
        fn dummy_ui() -> UI {todo!()}
        
        is_children(dummy_ui());
        is_children(Some(dummy_ui()));
        is_children(None::<UI>);
        is_children((1..=3).map(|_| dummy_ui()));
    }
    
    #[test]
    fn test_ui_new_unchecked() {
        assert_eq!(
            (unsafe {UI::new_unchecked(
                &[],
                []
            )}).0,
            r##""##
        );

        assert_eq!(
            (unsafe {UI::new_unchecked(
                &[
                    r##"<div>"##,
                ],
                []
            )}).0,
            r##"<div>"##
        );

        assert_eq!(
            (unsafe {UI::new_unchecked(
                &[
                    r##"<div class="##,
                    r##"></div>"##,
                ],
                [
                    Interpolator::Attribute(AttributeValue::from("foo")),
                ],
            )}).0,
            r##"<div class="foo"></div>"##
        );

        assert_eq!(
            (unsafe {UI::new_unchecked(
                &[
                    r##"<article class="##,
                    r##">"##,
                    r##"</article>"##,
                ],
                [
                    Interpolator::Attribute(AttributeValue::from("main-article")),
                    Interpolator::Children(IntoChildren::<_, true>::into_children(
                        (1..=3_usize).map(|i| UI::new_unchecked(
                            &[
                                r##"<p>i="##,
                                r##"</p>"##,
                            ],
                            [
                                Interpolator::Children(IntoChildren::<_, true>::into_children(
                                    i.to_string()
                                )),
                            ]
                        ))
                    )),
                ],
            )}).0,
            r##"<article class="main-article"><p>i=1</p><p>i=2</p><p>i=3</p></article>"##
        );
    }

    #[test]
    fn test_ui_interploate_expression() {
        let ui = UI! {
            {"an expression"}
        };
        assert_eq!(
            shoot(ui),
            r##"an expression"##
        );

        let ui = UI! {
            <p>"a text node"</p>
        };
        assert_eq!(
            shoot(ui),
            r##"<p>a text node</p>"##
        );

        let ui = UI! {
            <p>{"an expression"}</p>
        };
        assert_eq!(
            shoot(ui),
            r##"<p>an expression</p>"##
        );

        let ui = UI! {
            <div class="foo">
                <p>"hello"</p>
            </div>
        };
        let ui = UI! {
            <div class="bar">
                {ui}
            </div>
        };
        assert_eq!(
            shoot(ui),
            r##"<div class="bar"><div class="foo"><p>hello</p></div></div>"##
        );

        struct Layout {
            children: UI,
        }
        impl Beam for Layout {
            fn render(self) -> UI {
                UI! {
                    <html>
                        <head>
                            <meta charset="UTF-8">
                        </head>
                        <body>
                            {self.children}
                        </body>
                    </html>
                }
            }
        }

        assert_eq!(/* automatic doctype insertion */
            shoot(UI! { <Layout></Layout> }),
            r##"<!DOCTYPE html><html><head><meta charset="UTF-8"/></head><body></body></html>"##
        );

        assert_eq!(/* automatic doctype insertion */
            shoot(UI! { <Layout><h1>"Hello, Beam!"</h1></Layout> }),
            r##"<!DOCTYPE html><html><head><meta charset="UTF-8"/></head><body><h1>Hello, Beam!</h1></body></html>"##
        );

        let content = UI! {
            <h1>"Hello, Beam!"</h1>
        };
        assert_eq!(/* automatic doctype insertion */
            shoot(UI! { <Layout>{content}"[test]"</Layout> }),
            r##"<!DOCTYPE html><html><head><meta charset="UTF-8"/></head><body><h1>Hello, Beam!</h1>[test]</body></html>"##
        );
    }
}
