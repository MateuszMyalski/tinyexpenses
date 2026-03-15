use crate::auth::require_auth;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route("/edit/{year}", get(self::get::edit))
        .route("/edit/{year}", post(self::post::edit))
        .route("/create/{year}", get(self::get::create))
        .route("/create/{year}", post(self::post::create))
        .route_layer(axum::middleware::from_fn(require_auth))
}

pub mod get {
    use crate::account::Account;
    use crate::extraction;
    use crate::models::templates::{WebPageContext, WebPageToHtml, categories};
    use crate::models::views::ByYearDatabasePickerView;
    use crate::redirection;
    use axum::{Extension, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use tower_sessions::Session;

    pub async fn edit(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        let categories = match user.get_categories(year) {
            Ok(c) => c,
            Err(err) => {
                return redirection::by_error(err, year).into_response();
            }
        };

        let view = categories::EditTmpl {
            ctx: &WebPageContext::new("- Edit categories", &session)
                .await
                .with_messages(messages)
                .with_csrf(&csrf_token)
                .await,
            categories: categories.content(),
            picker: ByYearDatabasePickerView::new(
                &user.available_categories(),
                year,
                "/categories/edit/",
            ),
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn create(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        if user.get_categories(year).is_ok() {
            messages.warning("Tried to create category file for year that already has one.");
            return redirection::to_categories_edit(year).into_response();
        };

        let view = categories::CreateTmpl {
            ctx: &WebPageContext::new("- Create categories", &session)
                .await
                .with_messages(messages)
                .with_csrf(&csrf_token)
                .await,
            requested_year: year,
            picker: ByYearDatabasePickerView::new(&user.available_categories(), year, ""),
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

        let mut categories = match user.get_categories(year) {
            Ok(c) => c,
            Err(err) => {
                messages
                    .warning("Cannot edit categories file.")
                    .debug(err.to_string());
                return redirection::to_categories_edit(year).into_response();
            }
        };

        let Ok(data) = categories.from_json(&form.csv_content) else {
            messages.warning("Cannot deserialize categories.");
            return redirection::to_categories_edit(year).into_response();
        };

        categories.content_set(&data);

        if let Err(err) = categories.write() {
            messages
                .warning("Cannot overwrite categories.")
                .debug(err.to_string());
            return redirection::to_categories_edit(year).into_response();
        }

        messages.success("Categories file modified");

        return redirection::to_categories_edit(year).into_response();
    }

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
            if let Err(err) = user.create_categories(year) {
                error!("Cannot create categories file.");
                debug!("{}", err);
                messages.error("Unable to create categories file.");
                return redirection::to_categories_create(year).into_response();
            }
        } else {
            if let Err(err) = user.copy_categories(form.template_year, year) {
                error!("Cannot copy categories file.");
                debug!("{}", err);
                messages.error("Unable to copy category file.");
                return redirection::to_categories_create(year).into_response();
            }
        }

        messages.success("Category file created.");

        redirection::to_report_view(year).into_response()
    }
}
