//! Integrations with web frameworks
//!
//! # [Axum](https://github.com/tokio-rs/axum)
//! **Requires the `axum` crate feature.**
//!
//! Enables the [`struct@UI`] type to be returned as a response.
//! Its [`CONTENT_TYPE`] header is set to [`TEXT_HTML_UTF_8`].
//!
//! ```
//! use axum::{
//!     routing::get,
//!     Router,
//! };
//! use uibeam::UI;
//!
//! async fn handler() -> UI {
//!     UI! {
//!         <h1>"Hello, world!"</h1>
//!     }
//! }
//!
//! let app = Router::new()
//!     .route("/", get(handler));
//! # let _: Router = app;
//! ```

use http::{HeaderValue, header::CONTENT_TYPE};
use mime::TEXT_HTML_UTF_8;

use crate::UI;

#[cfg(feature = "axum")]
impl axum_core::response::IntoResponse for UI {
    fn into_response(self) -> axum_core::response::Response {
        let mut res = axum_core::body::Body::from(crate::shoot(self)).into_response();
        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static(TEXT_HTML_UTF_8.as_ref()),
        );
        res
    }
}
