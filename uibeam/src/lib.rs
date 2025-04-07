pub struct UI(std::borrow::Cow<'static, str>);
impl UI {
    #[doc(hidden)]
    /// tends to be used by the `UI!` macro internally.
    /// 
    /// ## SAFETY
    /// 
    /// 1. `template_pieces` must not be empty.
    /// 2. `template_pieces`'s length must be `N + 1`, where
    ///    `N` is the length of `expressions` to be interpolated.
    /// 3. `template_pieces` must `
    pub unsafe fn new_unchecked<const N: usize>(
        template_pieces: &'static [&'static str],
        expressions: [UI; N],
    ) -> Self {
        if template_pieces.len() - 1 != expressions.len() {
            // return Err(UIBeamError::InterpolationMismatch {
            //     expected: template_pieces.len() - 1,
            //     found: expressions.len(),
            // });
            unreachable!("UI! macro should ensure this never happens");
        }

        todo!()
    }
}
// `expressions` should be able to be either
// 
// 1. children UI(s)  : HTML nodes (including text nodes)
// 2. attribute value : escaped text value
// 
// so, current `[UI; N]` is not enough.

pub enum UIBeamError {
    Html(
        uibeam_html::Error,
    ),
    InterpolationMismatch {
        expected: usize,
        found: usize,
    },
}

pub enum Expression {
    AttributeValue(std::borrow::Cow<'static, str>),
    Children(Vec<UI>),
} // should not be an unified enum, but rather ...

pub trait Interpolator<X> {
    fn into_expression(self) -> Expression;
}

impl Interpolator<&'static str> for &'static str {
    fn into_expression(self) -> Expression {
        Expression::AttributeValue(std::borrow::Cow::Borrowed(self))
    }
}
impl Interpolator<String> for String {
    fn into_expression(self) -> Expression {
        Expression::AttributeValue(std::borrow::Cow::Owned(self))
    }
}
impl<T, X> Interpolator<Option<X>> for Option<T>
where
    T: Interpolator<X>,
{

}
impl<I> Interpolator<(I,)> for I
where
    I: Iterator<Item = UI>,
{}

#[cfg(test)]
fn assert_impls() {
    fn is_expression<T, E: Interpolation<T>>(_: E) {}
    fn dummy_ui() -> UI {todo!()}

    is_expression("Hello, UI Beam!");
    is_expression(format!("Hello, {}!", "UI Beam"));
    is_expression(Some("Hello, UI Beam!"));
    is_expression((1..=3).map(|_| dummy_ui()));
}
