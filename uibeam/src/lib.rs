pub struct UI(std::borrow::Cow<'static, str>);
impl UI {
    pub fn new<I>(
        template: &'static str,
        interpolation: impl Interpolations<I>,
    ) -> Result<Self, UIBeamError> {
        todo!()
    }
}

pub struct UIBeamError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// A collection of `Expression` types to complete the `UI` template
/// with interpolations.
pub trait Interpolations<I> {
    // fn interpolate(self, template: uibeam_html::Template) -> Result<UI, UIBeamError>;
}
impl Interpolations<()> for () {}

// impl<E1: Ex> Interpolations 

pub trait Expression<T> {
    // fn eval(self) -> Result<uibeam_html::Node, UIBeamError>;
}
impl Expression<&'static str> for &'static str {}
impl Expression<String> for String {}
// impl Expression for uibeam_html::Node {}
impl<I> Expression<(I,)> for I
where
    I: Iterator<Item = UI>,
{}
