use std::borrow::Cow;
use uibeam_html::html_escape;

pub use uibeam_macro::UI;

pub struct UI(Cow<'static, str>);

impl FromIterator<UI> for UI {
    #[inline]
    fn from_iter<T: IntoIterator<Item = UI>>(iter: T) -> Self {
        let mut result = String::new();
        for item in iter {
            result.push_str(&item.0);
        }
        UI(result.into())
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
impl UI {
    /// tends to be used by the `UI!` macro internally.
    /// 
    /// ## SAFETY
    /// 
    /// 1. `template_pieces` must have 0 or exactly `N + 1` pieces.
    /// 2. `template_pieces` must be concatenated into
    ///    a valid HTML string with any `interpolators` in place.
    /// 3. Each piece in `template_pieces` must be already HTML-escaped.
    ///    (intended to be escaped in `UI!` macro internally /
    ///     `new_unchecked` itself does not check or escape)
    pub unsafe fn new_unchecked<const N: usize>(
        template_pieces: &'static [&'static str],
        interpolators: [Interpolator; N],
    ) -> Self {
        UI(match template_pieces.len() {
            0 => Cow::Borrowed(""),
            1 => Cow::Borrowed(template_pieces[0]),
            _ => {
                let mut ui = String::from(template_pieces[0]);
                for i in 0..N {
                    match &interpolators[i] {
                        Interpolator::Children(children) => {
                            ui.push_str(&children.0);
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
                                assert!(ui.ends_with('='));
                            }
                            match value {
                                AttributeValue::Text(text) => {
                                    ui.push('"');
                                    ui.push_str(&html_escape(text));
                                    ui.push('"');
                                }
                                AttributeValue::Uint(uint) => {
                                    // here we don't need to escape
                                    ui.push('"');
                                    ui.push_str(&uint.to_string());
                                    ui.push('"');
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
                                    // this can be done by removing after the last whitespace of current `ui`
                                    // (because the SAFETY contract encusres `ui` is a part of a valid HTML string
                                    // and then at least one whitespace exists before an attribute name)
                                    let Some('=') = ui.pop() else {unreachable!()};
                                    if !*boolean {
                                        let Some(sp) = ui.rfind(is_ascii_whitespace) else {unreachable!()};
                                        ui.truncate(sp);
                                    }
                                    ui.push(' ');
                                }
                            }
                        }
                    }
                    ui.push_str(template_pieces[i + 1]);
                }
                Cow::Owned(ui)
            }
        })
    }
}

#[inline(always)]
pub const fn is_ascii_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0C' | '\r')
}

pub enum AttributeValue {
    Text(Cow<'static, str>),
    Uint(u64),
    Boolean(bool),
}
const _: () = {
    impl From<&'static str> for AttributeValue {
        fn from(value: &'static str) -> Self {
            AttributeValue::Text(value.into())
        }
    }
    impl From<String> for AttributeValue {
        fn from(value: String) -> Self {
            AttributeValue::Text(value.into())
        }
    }
    impl From<Cow<'static, str>> for AttributeValue {
        fn from(value: Cow<'static, str>) -> Self {
            AttributeValue::Text(value)
        }
    }

    impl From<bool> for AttributeValue {
        fn from(value: bool) -> Self {
            AttributeValue::Boolean(value)
        }
    }

    macro_rules! uint_attribute_values {
        ($($t:ty),+) => {
            $(
                impl From<$t> for AttributeValue {
                    fn from(it: $t) -> Self {
                        AttributeValue::Uint(it.into())
                    }
                }
            )+
        };
    }
    uint_attribute_values!(u8, u16, u32, u64);

    impl From<usize> for AttributeValue {
        fn from(it: usize) -> Self {
            if cfg!(any(
                target_pointer_width = "16",
                target_pointer_width = "32",
                target_pointer_width = "64",
            )) {
                AttributeValue::Uint(it as u64)
            } else {
                unreachable!("UIBeam does not support 128-bit CPU architectures");
            }
        }
    }
};

pub trait IntoChildren<T> {
    fn into_children(self) -> UI;
}
const _: () = {
    impl IntoChildren<UI> for UI {
        fn into_children(self) -> UI {
            self
        }
    }

    // note that `Option<UI>` implements `IntoChildren` because `Option` is `IntoIterator`
    impl<I> IntoChildren<(I,)> for I
    where
        I: IntoIterator<Item = UI>,
    {
        fn into_children(self) -> UI {
            UI::from_iter(self)
        }
    }

    impl<D: std::fmt::Display> IntoChildren<&dyn std::fmt::Display> for D {
        fn into_children(self) -> UI {
            let s = self.to_string();
            match html_escape(&s) {
                Cow::Owned(escaped) => {
                    UI(Cow::Owned(escaped))
                }
                Cow::Borrowed(_) => {
                    // this means `s` is already escaped, so we can avoid allocation,
                    // just using `s` directly
                    UI(Cow::Owned(s))
                }
            }
        }
    }
};

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
    fn test_html_escape() {
        let test_cases = [
            ("", ""),
            ("abc", "abc"),
            ("おはよう", "おはよう"),
            ("&", "&amp;"),
            ("<", "&lt;"),
            (">", "&gt;"),
            ("\"", "&#34;"),
            ("'", "&#39;"),
            (
                "a&b<c>d\"'e",
                "a&amp;b&lt;c&gt;d&#34;&#39;e"
            ),
            (
                "a&b<c>d\"'e&f<g>h\"'i",
                "a&amp;b&lt;c&gt;d&#34;&#39;e&amp;f&lt;g&gt;h&#34;&#39;i"
            ),
            (
                "flowers <script>evil_script()</script>",
                "flowers &lt;script&gt;evil_script()&lt;/script&gt;"
            ),
            (
                "こんにちは <script>console.alert('ぼくはまちちゃん')</script>",
                "こんにちは &lt;script&gt;console.alert(&#39;ぼくはまちちゃん&#39;)&lt;/script&gt;"
            ),
        ];
        
        for (input, expected) in test_cases {
            assert_eq!(html_escape(input), expected);
        }
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
                    Interpolator::Children(IntoChildren::into_children(
                        (1..=3_usize).map(|i| UI::new_unchecked(
                            &[
                                r##"<p>i="##,
                                r##"</p>"##,
                            ],
                            [
                                Interpolator::Children(IntoChildren::into_children(
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
}
