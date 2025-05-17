use crate::{UI, shoot};

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for UI {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        axum::response::Html(shoot(self)).into_response()
    }
}

#[cfg(feature = "actix-web")]
impl actix_web::Responder for UI {
    type Body = <actix_web::web::Html as actix_web::Responder>::Body;

    #[inline]
    fn respond_to(self, req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        actix_web::web::Html::new(shoot(self)).respond_to(req)
    }
}
