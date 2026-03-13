use crate::auth::require_api_auth;
use axum::{Router, routing::put};

pub fn router() -> Router {
    Router::new()
        .route(
            "/v1/{username}/report/append",
            put(self::put::report_append),
        )
        .route_layer(axum::middleware::from_fn(require_api_auth))
}

pub mod put {
    use crate::account::Account;
    use crate::auth::ApiResponsePayload;
    use crate::models::csv::types::ReportEntry;
    use crate::models::templates::report;
    use crate::storage::AppendableRepository;
    use axum::{Extension, extract, response::IntoResponse};
    use chrono::Datelike;
    use tracing::log::{debug, error};

    pub async fn report_append(
        Extension(user): Extension<Account>,
        extract::Json(payload): extract::Json<report::AppendApi>,
    ) -> impl IntoResponse {
        let Ok(mut report) = user.get_report(payload.date.year()) else {
            return ApiResponsePayload::new().as_nok();
        };

        let report_entry = ReportEntry::new()
            .with_date(&payload.date)
            .with_curr_timestamp()
            .with_description(&payload.description.unwrap_or_default())
            // We need to parse value from string to not overcomplicated extractor, some apps can format
            // number with comma separators, some with dot.
            .with_value(payload.value.trim().replace(',', ".").parse().unwrap_or_default())
            .with_subcategory(&payload.subcategory);

        if let Err(err) = report.append(&report_entry) {
            error!("Cannot store entry to report during API request.");
            debug!("{}", err);
            ApiResponsePayload::new().as_nok();
        }

        ApiResponsePayload::new().as_ok()
    }
}
