use std::borrow::Cow;

/// Escapes HTML special characters in a string.
///
/// This function replaces the following characters with their HTML
/// entity equivalents:
///
/// * `&` -> `&amp;`
/// * `<` -> `&lt;`
/// * `>` -> `&gt;`
/// * `"` -> `&#34;`
/// * `'` -> `&#39;`
///
/// When the input string does not contain any special characters,
/// it just returns a borrowed reference to the original string.
#[inline]
pub fn escape(s: &str) -> Cow<'_, str> {
    let mut first_special = None;
    for i in 0..s.len() {
        match &s.as_bytes()[i] {
            b'&' | b'<' | b'>' | b'"' | b'\'' => {
                first_special = Some(i);
                break;
            }
            _ => (),
        }
    }

    match first_special {
        None => Cow::Borrowed(s),
        Some(f) => {
            let mut escaped = Vec::with_capacity(s.len() + 10);
            escaped.extend_from_slice(&s.as_bytes()[..f]);
            for b in &s.as_bytes()[f..] {
                match b {
                    b'&' => escaped.extend_from_slice(b"&amp;"),
                    b'<' => escaped.extend_from_slice(b"&lt;"),
                    b'>' => escaped.extend_from_slice(b"&gt;"),
                    b'"' => escaped.extend_from_slice(b"&#34;"), // "&#34;" is shorter than "&quot;".
                    b'\'' => escaped.extend_from_slice(b"&#39;"), // "&#39;" is shorter than "&apos;" and apos was not in HTML until HTML5.
                    _ => escaped.push(*b),                        // no need to escape.
                                                                   // this may make `escaped` invalid UTF-8 **temporarily**, but finally it **builds** a valid UTF-8 bytes.
                }
            }
            // SAFETY: `escaped` is a valid UTF-8 bytes because:
            //
            // - original `s` is valid UTF-8
            // - we just replaced some ASCII bytes with valid UTF-8 bytes
            // - the rest of `escaped` is unchanged, directly copied from `s`
            Cow::Owned(unsafe { String::from_utf8_unchecked(escaped) })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape() {
        let test_cases = [
            ("", ""),
            ("abc", "abc"),
            ("おはよう", "おはよう"),
            ("&", "&amp;"),
            ("<", "&lt;"),
            (">", "&gt;"),
            ("\"", "&#34;"),
            ("'", "&#39;"),
            ("a&b<c>d\"'e", "a&amp;b&lt;c&gt;d&#34;&#39;e"),
            (
                "a&b<c>d\"'e&f<g>h\"'i",
                "a&amp;b&lt;c&gt;d&#34;&#39;e&amp;f&lt;g&gt;h&#34;&#39;i",
            ),
            (
                "flowers <script>evil_script()</script>",
                "flowers &lt;script&gt;evil_script()&lt;/script&gt;",
            ),
            (
                "こんにちは <script>console.alert('ぼくはまちちゃん')</script>",
                "こんにちは &lt;script&gt;console.alert(&#39;ぼくはまちちゃん&#39;)&lt;/script&gt;",
            ),
        ];

        for (input, expected) in test_cases {
            assert_eq!(escape(input), expected);
        }
    }
}
