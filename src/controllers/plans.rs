use crate::auth::require_auth;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route("/view/{year}", get(self::get::view))
        .route("/create/{year}", get(self::get::create))
        .route("/create/{year}", post(self::post::create))
        .route("/edit/{year}", get(self::get::edit))
        .route("/edit/{year}", post(self::post::edit))
        .route_layer(axum::middleware::from_fn(require_auth))
}

pub mod get {
    use crate::account::Account;
    use crate::budget::Budget;
    use crate::models::templates::{WebPageContext, WebPageToHtml, plans};
    use crate::models::views::{self, ByYearDatabasePickerView, PlansSummaryView};
    use crate::{extraction, redirection};
    use axum::{Extension, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use chrono::NaiveDate;
    use tower_sessions::Session;

    pub async fn create(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        if user.get_plans(year).is_ok() {
            messages.warning("Tried to create budget plans file for year that already has one.");
            return redirection::to_plans_edit(year).into_response();
        };

        let view = plans::CreateTmpl {
            ctx: &WebPageContext::new("- Create budget plans")
                .with_messages(messages)
                .with_csrf(&csrf_token, &session)
                .await,
            requested_year: year,
            picker: ByYearDatabasePickerView::new(&user.available_plans(), year, ""),
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn view(
        _csrf_token: CsrfToken,
        messages: Messages,
        _session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        let categories = match user.get_categories(year) {
            Ok(c) => c,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let plans = match user.get_plans(year) {
            Ok(r) => r,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let mut budget = Budget::new(&categories);
        budget.load_from_plans(&plans.content());

        let title = format!("- Yearly Budget plans of {year}");

        let view = plans::ViewTmpl {
            ctx: &WebPageContext::new(&title)
                .with_messages(messages)
                .with_account(&user),
            view_date: NaiveDate::from_ymd_opt(year, 01, 01)
                .expect("Hardcoded date just to generate labels."),
            picker: ByYearDatabasePickerView::new(&user.available_reports(), year, "/plans/view/"),
            table: PlansSummaryView::new(&budget, &categories),
        };

        view.to_html().into_response()
    }

    pub async fn edit(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        let Ok(plans) = user.get_plans(year) else {
            return redirection::to_plans_create(year).into_response();
        };

        let view = plans::EditTmpl {
            ctx: &WebPageContext::new("- Edit budget plans")
                .with_messages(messages)
                .with_csrf(&csrf_token, &session)
                .await,
            entries: plans.content(),
            months_labels: views::MONTHS,
            picker: ByYearDatabasePickerView::new(&user.available_plans(), year, "/plans/edit/"),
        }
        .to_html();

        (csrf_token, view).into_response()
    }
}

pub mod post {
    use crate::account::Account;
    use crate::csrf;
    use crate::extraction;
    use crate::models::templates;
    use crate::redirection;
    use crate::storage::BasicRepository;
    use axum::{Extension, Form, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use tower_sessions::Session;
    use tracing::log::{debug, error};

    pub async fn create(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
        Form(form): Form<templates::WithTemplateCreateForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        if year == form.template_year {
            if let Err(err) = user.create_plans(year) {
                error!("Cannot create budget plans file.");
                debug!("{}", err);
                messages.error("Unable to create budget plans file.");
                return redirection::to_plans_create(year).into_response();
            }
        } else {
            if let Err(err) = user.copy_plans(form.template_year, year) {
                error!("Cannot copy budget plans file.");
                debug!("{}", err);
                messages.error("Unable to copy budget plans file.");
                return redirection::to_plans_create(year).into_response();
            }
        }

        messages.success("Budget plans file created.");

        redirection::to_plans_view(year).into_response()
    }

    pub async fn edit(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
        Form(form): Form<templates::CommonCsvEditForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        let mut plans = match user.get_plans(year) {
            Ok(c) => c,
            Err(err) => {
                messages
                    .warning("Cannot edit budget plans file.")
                    .debug(err.to_string());
                return redirection::to_plans_edit(year).into_response();
            }
        };

        let Ok(data) = plans.from_json(&form.csv_content) else {
            messages.warning("Cannot deserialize budget plans.");
            return redirection::to_plans_edit(year).into_response();
        };

        plans.content_set(&data);

        if let Err(err) = plans.write() {
            messages
                .warning("Cannot overwrite budget plans.")
                .debug(err.to_string());
            return redirection::to_plans_edit(year).into_response();
        }

        messages.success("Budget plans file modified");

        return redirection::to_plans_view(year).into_response();
    }
}
