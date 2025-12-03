use crate::{UI, shoot};

#[cfg(feature = "axum")]
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
impl axum_core::response::IntoResponse for UI {
    #[inline]
    fn into_response(self) -> axum_core::response::Response {
        // ref: https://github.com/tokio-rs/axum/blob/6ad76dd9a4c07012044845b026ad17ad8de2a9bd/axum/src/response/mod.rs#L38-L52
        axum_core::response::IntoResponse::into_response((
            [(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
            )],
            shoot(self),
        ))
    }
}

#[cfg(feature = "actix-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix-web")))]
impl actix_web::Responder for UI {
    type Body = <actix_web::web::Html as actix_web::Responder>::Body;

    #[inline]
    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        actix_web::web::Html::new(shoot(self)).respond_to(req)
    }
}

#[cfg(feature = "ohkami")]
#[cfg_attr(docsrs, doc(cfg(feature = "ohkami")))]
impl ohkami::claw::content::IntoContent for UI {
    const CONTENT_TYPE: &'static str =
        <ohkami::claw::content::Html as ohkami::claw::content::IntoContent>::CONTENT_TYPE;

    #[inline]
    fn into_content(self) -> Result<std::borrow::Cow<'static, [u8]>, impl std::fmt::Display> {
        ohkami::claw::content::IntoContent::into_content(ohkami::claw::content::Html(shoot(self)))
    }

    #[cfg(feature = "openapi")]
    fn openapi_responsebody() -> impl Into<ohkami::openapi::schema::SchemaRef> {
        <ohkami::claw::content::Html as ohkami::claw::content::IntoContent>::openapi_responsebody()
    }
}
