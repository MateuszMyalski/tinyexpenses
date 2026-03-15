use crate::auth::require_auth;
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/", get(self::get::index))
        .route_layer(axum::middleware::from_fn(require_auth))
        .route("/error", get(self::get::error))
}

pub mod get {
    use crate::models::templates::{WebPageContext, WebPageToHtml, error};
    use crate::redirection;
    use axum::response::IntoResponse;
    use axum_messages::Messages;
    use tower_sessions::Session;

    pub async fn index() -> impl IntoResponse {
        redirection::to_dashboard()
    }

    pub async fn error(messages: Messages, session: Session) -> impl IntoResponse {
        error::UrlTmpl {
            ctx: &WebPageContext::new("- URL Error", &session)
                .await
                .with_messages(messages),
        }
        .to_html()
        .into_response()
    }
}
