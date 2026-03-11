use crate::auth::require_auth;
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route("/create/{year}", get(self::get::create))
        .route("/create/{year}", post(self::post::create))
        .route("/append", get(self::get::append))
        .route("/append", post(self::post::append))
        .route("/edit/{year}", get(self::get::edit))
        .route("/edit/{year}", post(self::post::edit))
        .route("/view/{year}/{month}", get(self::get::view_month))
        .route("/view/{year}", get(self::get::view_year))
        .route_layer(axum::middleware::from_fn(require_auth))
}

pub mod get {
    use crate::account::Account;
    use crate::budget::Budget;
    use crate::extraction;
    use crate::models::csv::tables::Categories;
    use crate::models::templates::{WebPageContext, WebPageToHtml, report};
    use crate::models::views::{
        ByMonthDatabasePickerView, ByYearDatabasePickerView, MonthReportSummaryView,
        YearReportSummaryView,
    };
    use crate::redirection;
    use axum::{Extension, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use chrono::{Datelike, Local, NaiveDate};
    use tower_sessions::Session;

    pub async fn create(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        if user.available_reports().contains(&year) {
            return redirection::to_report_view(year).into_response();
        }

        let view = report::ViewCreateReportTmpl {
            ctx: &WebPageContext::new("- Create report")
                .with_messages(messages)
                .with_csrf(&csrf_token, &session)
                .await,
            year,
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn append(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
    ) -> impl IntoResponse {
        let now = Local::now();

        let Ok(categories) = user.get_categories(now.year()) else {
            return redirection::to_categories_create(now.year()).into_response();
        };

        let view = report::ViewAppendTmpl {
            ctx: &WebPageContext::new("- Append report")
                .with_messages(messages)
                .with_csrf(&csrf_token, &session)
                .await,
            types_with_categories: categories.map_by_category().into_iter().collect(),
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn view_month(
        messages: Messages,
        Extension(user): Extension<Account>,
        extraction::Path((year, month)): extraction::Path<(i32, u32)>,
    ) -> impl IntoResponse {
        if month < 1 || month > 12 {
            messages.error("Invalid month number, range is valid only for numbers 1..12.");
            return redirection::to_index().into_response();
        }

        let categories = match user.get_categories(year) {
            Ok(c) => c,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let report = match user.get_report(year) {
            Ok(r) => r,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let mut budget = Budget::new(&categories);
        budget.load_from_report(&report.content());

        let mut table_view =
            MonthReportSummaryView::new(&budget, &categories, (month - 1) as usize);

        if let Ok(plans) = user.get_plans(year) {
            let category_map = categories.map_by_category();

            table_view.compare_with_plans(
                &plans.content().as_slice(),
                category_map
                    .get(Categories::INCOME_LABEL)
                    .map_or(&[] as &[String], Vec::as_slice),
            );
        }

        let title = format!("-Monthly Budget of {year}-{month:02}");

        let view = report::ViewMonthTmpl {
            ctx: &WebPageContext::new(&title)
                .with_messages(messages)
                .with_account(&user),
            view_date: NaiveDate::from_ymd_opt(year, month, 1)
                .expect("Hardcoded date must be valid"),
            picker_year: ByYearDatabasePickerView::new(
                &user.available_plans(),
                year,
                "/report/view/",
            ),
            picker_month: ByMonthDatabasePickerView::new(year, month as i32, "/report/view/"),
            table: table_view,
        };

        view.to_html().into_response()
    }

    pub async fn view_year(
        messages: Messages,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
    ) -> impl IntoResponse {
        let categories = match user.get_categories(year) {
            Ok(c) => c,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let report = match user.get_report(year) {
            Ok(r) => r,
            Err(err) => return redirection::by_error(err, year).into_response(),
        };

        let mut budget = Budget::new(&categories);
        budget.load_from_report(&report.content());

        let mut table_view = YearReportSummaryView::new(&budget, &categories);

        if let Ok(plans) = user.get_plans(year) {
            let category_map = categories.map_by_category();

            table_view.compare_with_plans(
                &plans.content().as_slice(),
                category_map
                    .get(Categories::INCOME_LABEL)
                    .map_or(&[] as &[String], Vec::as_slice),
            );
        }

        let title = format!("- Yearly Budget of {year}");

        let view = report::ViewYearTmpl {
            ctx: WebPageContext::new(&title)
                .with_messages(messages)
                .with_account(&user),
            picker: ByYearDatabasePickerView::new(&user.available_plans(), year, "/report/view/"),
            table: table_view,
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
        let Ok(report) = user.get_report(year) else {
            return redirection::to_report_create(year).into_response();
        };

        let view = report::EditTmpl {
            ctx: &WebPageContext::new("- Edit report")
                .with_messages(messages)
                .with_csrf(&csrf_token, &session)
                .await,
            entries: report.content(),
            picker: ByYearDatabasePickerView::new(&user.available_plans(), year, "/report/edit/"),
        }
        .to_html();

        (csrf_token, view).into_response()
    }
}

pub mod post {
    use crate::account::Account;
    use crate::csrf;
    use crate::extraction;
    use crate::models::csv::tables::Categories;
    use crate::models::csv::types::ReportEntry;
    use crate::models::templates;
    use crate::models::templates::report;
    use crate::redirection;
    use crate::storage::{AppendableRepository, BasicRepository};
    use axum::{Extension, Form, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use chrono::Datelike;
    use tower_sessions::Session;

    pub async fn create(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        extraction::Path(year): extraction::Path<i32>,
        Form(form): Form<report::CreateForm>,
    ) -> impl IntoResponse {
        if !crate::csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        if let Err(err) = user.create_report(year) {
            messages
                .error("Cannot create report - error while creating storage.")
                .debug(err.to_string());
            return redirection::to_report_create(year).into_response();
        }

        let mut report = match user.get_report(year) {
            Ok(r) => r,
            Err(err) => {
                messages
                    .error("Unable to load report file.")
                    .debug(err.to_string());
                return redirection::to_report_create(year).into_response();
            }
        };

        let report_record = ReportEntry::new()
            .with_curr_date()
            .with_curr_timestamp()
            .with_subcategory(Categories::INITIAL_BALANCE_LABEL)
            .with_value(form.initial_balance);

        if let Err(err) = report.append(&report_record) {
            messages
                .error("Unable to insert initial balance to report.")
                .debug(err.to_string());
            return redirection::to_report_create(year).into_response();
        }

        messages.success("Report created.");
        redirection::to_report_view(year).into_response()
    }

    pub async fn append(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        Form(form): Form<report::AppendForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        let mut report = match user.get_report(form.date.year()) {
            Ok(r) => r,

            Err(err) => {
                messages
                    .error("Unable to load report file.")
                    .debug(err.to_string());
                return redirection::to_report_create(form.date.year()).into_response();
            }
        };

        let report_record = ReportEntry::new()
            .with_curr_timestamp()
            .with_date(&form.date)
            .with_subcategory(&form.subcategory)
            .with_value(form.value)
            .with_description(&form.description);

        if let Err(err) = report.append(&report_record) {
            messages
                .error("Unable to insert entry to report.")
                .debug(err.to_string());
            return redirection::to_report_append().into_response();
        }

        let categories = match user.get_categories(form.date.year()) {
            Ok(r) => r,
            Err(err) => {
                messages
                    .error("Unable to load categories file.")
                    .debug(err.to_string());
                return redirection::to_report_create(form.date.year()).into_response();
            }
        };

        let mut are_savings_updated = false;
        if categories.is_savings(&form.subcategory) {
            let mut savings = user.get_savings();

            savings
                .update(&form.subcategory)
                .value
                .change_by(form.value);

            if let Err(err) = savings.write() {
                messages
                    .success("Entry appended to report.")
                    .error("Unable to update savings.")
                    .debug(err.to_string());
                return redirection::to_report_append().into_response();
            } else {
                are_savings_updated = true;
            }
        }

        let messages = messages.success("Entry appended to report.");
        are_savings_updated.then(|| messages.info("Savings updated."));

        redirection::to_report_append().into_response()
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

        let mut report = match user.get_report(year) {
            Ok(r) => r,
            Err(err) => {
                messages
                    .warning("Cannot edit report file.")
                    .debug(err.to_string());
                return redirection::to_report_edit(year).into_response();
            }
        };

        let Ok(data) = report.from_json(&form.csv_content) else {
            messages.warning("Cannot deserialize report entries.");
            return redirection::to_report_edit(year).into_response();
        };

        report.content_set(&data);

        if let Err(err) = report.write() {
            messages
                .warning("Cannot overwrite report.")
                .debug(err.to_string());
        } else {
            messages.success("Report file modified");
        }

        return redirection::to_report_edit(year).into_response();
    }
}
