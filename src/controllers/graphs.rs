use crate::auth::require_auth;
use axum::{Router, routing::get};

pub fn router() -> Router {
    Router::new()
        .route("/view/{year}", get(self::get::view))
        .route_layer(axum::middleware::from_fn(require_auth))
}

pub mod get {
    use crate::budget::Budget;
    use crate::extraction;
    use crate::models::templates::WebPageContext;
    use crate::models::templates::graphs::GraphsYearTmpl;
    use crate::models::views::{ByYearDatabasePickerView, MONTHS, YearGraphView};
    use crate::redirection;
    use crate::{account::Account, models::templates::WebPageToHtml};
    use axum::{Extension, response::IntoResponse};
    use axum_messages::Messages;
    use tower_sessions::Session;

    pub async fn view(
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        let report = match user.get_report(year) {
            Ok(r) => r,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let categories = match user.get_categories(year) {
            Ok(r) => r,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let mut budget = Budget::new(&categories);
        budget.load_from_report(&report.content());

        let view = GraphsYearTmpl {
            ctx: &WebPageContext::new("- View graph year details", &session)
                .await
                .with_messages(messages)
                .with_account(&user),
            graph: YearGraphView::new(&budget, &categories),
            months_label: MONTHS,
            picker: ByYearDatabasePickerView::new(&user.available_reports(), year, "/graphs/view/"),
        }
        .to_html();

        view.into_response()
    }
}
