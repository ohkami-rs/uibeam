use std::borrow::Cow;

pub struct UI(Cow<'static, str>);

impl FromIterator<UI> for UI {
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
    Attribute(AttributeValue),
    Children(UI),
}

impl UI {
    #[doc(hidden)]
    /// tends to be used by the `UI!` macro internally.
    /// 
    /// ## SAFETY
    /// 
    /// 1. `template_pieces` must have 0 or exactly `N + 1` pieces.
    /// 2. `template_pieces` must be concatenated into
    ///   a valid HTML string with any `interpolators` in place.
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

// referencing Go standard library: <https://github.com/golang/go/blob/a2c959fe97e094f337f46e529e9e7d1a34a7c26a/src/html/escape.go#L166-L180>
pub fn html_escape(s: &str) -> Cow<'_, str> {
    let mut first_special = None;
    for i in 0..s.len() {
        match &s.as_bytes()[i] {
            b'&' | b'<' | b'>' | b'"' | b'\'' => {
                first_special = Some(i);
                break;
            }
            _ => ()
        }
    }

    match first_special {
        None => {
            Cow::Borrowed(s)
        }
        Some(f) => {
            let mut escaped = Vec::with_capacity(s.len() + 10);
            escaped.extend_from_slice(&s.as_bytes()[..f]);
            for b in &s.as_bytes()[f..] {
                match b {
                    b'&'  => escaped.extend_from_slice(b"&amp;"),
                    b'<'  => escaped.extend_from_slice(b"&lt;"),
                    b'>'  => escaped.extend_from_slice(b"&gt;"),
                    b'"'  => escaped.extend_from_slice(b"&#34;"), // "&#34;" is shorter than "&quot;".
                    b'\'' => escaped.extend_from_slice(b"&#39;"), // "&#39;" is shorter than "&apos;" and apos was not in HTML until HTML5.
                    _ => escaped.push(*b), // no need to escape.
                    // this may make `escaped` invalid UTF-8 **temporarily**, but finally it **builds** a valid UTF-8 bytes.
                }
            }
            // SAFETY: `escaped` is a valid UTF-8 bytes because:
            // 
            // - original `s` is valid UTF-8
            // - we just replaced some ascii bytes with valid UTF-8 bytes
            // - the rest of `escaped` is unchanged, directly copied from `s`
            Cow::Owned(unsafe {String::from_utf8_unchecked(escaped)})
        }
    }
}

pub enum UIBeamError {
    Html(
        uibeam_html::Error,
    ),
    InterpolationMismatch {
        expected: usize,
        found: usize,
    },
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

pub trait IntoChildren {
    fn into_children(self) -> UI;
}
const _: () = {
    impl IntoChildren for UI {
        fn into_children(self) -> UI {
            self
        }
    }

    // note `Option<UI>` implements `IntoChildren` because `Option` is `IntoIterator`
    impl<I> IntoChildren for I
    where
        I: IntoIterator<Item = UI>,
    {
        fn into_children(self) -> UI {
            UI::from_iter(self)
        }
    }
};

#[cfg(test)]
mod test {
    use super::*;

    /* compiles */
    fn __assert_impls__() {
        fn is_children<C: IntoChildren>(_: C) {}
        
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
                                Interpolator::Attribute(AttributeValue::from(i)),
                            ]
                        ))
                    )),
                ],
            )}).0,
            r##"<article class="main-article"><p>i=1</p><p>i=2</p><p>i=3</p></article>"##
        );
    }
}
