pub enum Error {}

/// incomplete HTML nodes, maybe including placeholders to render Rust expressions
pub struct Template {
    // todo
}

pub enum Node {
    Text(std::borrow::Cow<'static, str>),
}

// pub fn 
