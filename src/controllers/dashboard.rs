use crate::auth::require_auth;
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/", get(self::get::root))
        .route_layer(axum::middleware::from_fn(require_auth))
}
pub mod get {
    use crate::models::templates::{WebPageContext, WebPageToHtml, dashboard};
    use axum::response::IntoResponse;
    use axum_messages::Messages;

    pub async fn root(messages: Messages) -> impl IntoResponse {
        dashboard::Tmpl {
            ctx: &WebPageContext::new("- Dashboard").with_messages(messages),
        }
        .to_html()
    }
}
