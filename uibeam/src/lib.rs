pub struct UI(std::borrow::Cow<'static, str>);

impl FromIterator<UI> for UI {
    fn from_iter<T: IntoIterator<Item = UI>>(iter: T) -> Self {
        let mut result = String::new();
        for item in iter {
            result.push_str(&item.0);
        }
        UI(result.into())
    }
}

const _: () = {
    #[doc(hidden)]
    pub enum Interpolator {
        AttributeValue(AttributeValue),
        Children(UI),
    }

    impl UI {
        #[doc(hidden)]
        /// tends to be used by the `UI!` macro internally.
        /// 
        /// ## SAFETY
        /// 
        /// 1. `template_pieces` must not be empty.
        /// 2. `template_pieces`'s length must be `N + 1`, where
        ///    `N` is the length of `expressions` to be interpolated.
        /// 3. `template_pieces` must be concatenated into
        ///     a valid HTML string with any `interpolators` in place.
        pub unsafe fn new_unchecked<const N: usize>(
            template_pieces: &'static [&'static str],
            interpolators: [Interpolator; N],
        ) -> Self {
            #[cfg(debug_assertions)]
            if template_pieces.len() - 1 != interpolators.len() {
                unreachable!("UI! macro should ensure this never happens");
            }

            if template_pieces.len() == 1 {
                return UI(template_pieces[0].into());
            }

            let mut ui = String::from(template_pieces[0]);
            for i in 0..N {
                match &interpolators[i] {
                    Interpolator::AttributeValue(value) => {
                        // FIXME:
                        // in this way we CANNOT skip an attribute when
                        // `value` is `Boolean(false)`!!!
                        ui.push('"');
                        ui.push_str(todo!());
                        ui.push('"');
                    }
                    Interpolator::Children(children) => {
                        ui.push_str(&children.0);
                    }
                }
                ui.push_str(template_pieces[i + 1]);
            }
            UI(ui.into())
        }
    }
};

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
    Text(std::borrow::Cow<'static, str>),
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

pub trait IntoChildren<X> {
    fn into_children(self) -> UI;
}
const _: () = {
    impl IntoChildren<UI> for UI {
        fn into_children(self) -> UI {
            self
        }
    }
    impl IntoChildren<Option<UI>> for Option<UI> {
        fn into_children(self) -> UI {
            UI::from_iter(self.into_iter())
        }
    }
    impl<I> IntoChildren<(I,)> for I
    where
        I: Iterator<Item = UI>,
    {
        fn into_children(self) -> UI {
            UI::from_iter(self)
        }
    }
};

#[cfg(test)]
fn __assert_impls__() {
    fn is_children<X, C: IntoChildren<X>>(_: C) {}

    fn dummy_ui() -> UI {todo!()}

    is_children(dummy_ui());
    is_children(Some(dummy_ui()));
    is_children(None::<UI>);
    is_children((1..=3).map(|_| dummy_ui()));
}
