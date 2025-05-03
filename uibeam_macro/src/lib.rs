macro_rules! joined_span {
    ($span:expr $( , $other_span:expr )* $(,)?) => {
        {
            let mut span: proc_macro2::Span = $span;
            $(
                let other_span: Option<proc_macro2::Span> = $other_span.into();
                if let Some(other_span) = other_span {
                    span = span.join(other_span).unwrap_or(span);
                }
            )+
            span
        }
    };
}

mod ui;

#[proc_macro]
#[allow(non_snake_case)]
pub fn UI(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ui::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
