use crate::{UI, shoot};

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for UI {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        axum::response::Html(shoot(self)).into_response()
    }
}
