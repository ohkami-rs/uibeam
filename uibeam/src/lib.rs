pub struct UI(std::borrow::Cow<'static, str>);
impl UI {
    pub fn new(
        template: &'static str,
        interpolation: impl Interpolation,
    ) -> Result<Self, UIBeamError> {
        todo!()
    }
}

pub struct UIBeamError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

pub trait Interpolation {
    fn interpolate(self, template: uibeam_html::Template) -> Result<UI, UIBeamError>;
}

pub trait Expression {

}
