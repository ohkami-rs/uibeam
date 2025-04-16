use std::borrow::Cow;

#[inline]
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
