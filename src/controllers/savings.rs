use crate::auth::require_auth;
use axum::{
    Router, middleware,
    routing::{get, post},
};

pub fn router() -> Router {
    Router::new()
        .route("/edit", get(self::get::edit))
        .route("/edit", post(self::post::edit))
        .route("/withdraw", get(self::get::withdraw))
        .route("/withdraw", post(self::post::withdraw))
        .route("/view", get(self::get::view))
        .route_layer(middleware::from_fn(require_auth))
}

pub mod get {
    use crate::account::Account;
    use crate::models::resources;
    use crate::models::templates::{WebPageContext, WebPageToHtml, savings};
    use crate::models::views::ByYearDatabasePickerView;
    use crate::redirection;
    use crate::savings::SavingsCollection;
    use axum::Form;
    use axum::{Extension, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;
    use chrono::{Datelike, Utc};
    use tower_sessions::Session;

    pub async fn edit(
        csrf_token: CsrfToken,
        Extension(user): Extension<Account>,
        messages: Messages,
        session: Session,
        Form(form): Form<savings::ViewForm>,
    ) -> impl IntoResponse {
        let savings = user.get_savings();

        let entry = match savings.get(&form.subcategory) {
            Some(e) => e,
            None => {
                messages.warning(format!(
                    "Saving for '{}' in '{}' not found.",
                    form.subcategory, form.account
                ));
                return redirection::to_savings_view().into_response();
            }
        };

        let view = savings::EditTmpl {
            ctx: &WebPageContext::new("- Edit savings", &session)
                .await
                .with_messages(messages)
                .with_csrf(&csrf_token)
                .await,
            account: entry.account_name.get(),
            balance: entry.value.get(),
            subcategory: entry.subcategory.get(),
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn withdraw(
        csrf_token: CsrfToken,
        Extension(user): Extension<Account>,
        messages: Messages,
        session: Session,
        Form(form): Form<savings::ViewForm>,
    ) -> impl IntoResponse {
        let savings = user.get_savings();

        let entry = match savings.get(&form.subcategory) {
            Some(e) => e,
            None => {
                messages.warning(format!(
                    "Saving for '{}' in '{}' not found.",
                    form.subcategory, form.account
                ));
                return redirection::to_savings_view().into_response();
            }
        };

        let view = savings::WithdrawTmpl {
            ctx: &WebPageContext::new("- Withdraw savings", &session)
                .await
                .with_messages(messages)
                .with_account(&user)
                .with_csrf(&csrf_token)
                .await,
            subcategory: entry.subcategory.get().to_string(),
            account: entry.account_name.get().to_string(),
            balance: entry.value.get(),
            picker: ByYearDatabasePickerView::new(
                &user.available_reports(),
                Utc::now().date_naive().year(),
                "",
            ),
        }
        .to_html();

        (csrf_token, view).into_response()
    }

    pub async fn view(
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
    ) -> impl IntoResponse {
        let savings = user.get_savings();

        let savings_collection = SavingsCollection::new(&savings.content().as_slice());
        let mut tables = Vec::new();
        for (label, records) in savings_collection.iter() {
            tables.push(resources::SavingsAccount::new(label, records));
        }

        let view = savings::ViewTmpl {
            ctx: &WebPageContext::new("- View savings", &session)
                .await
                .with_messages(messages)
                .with_account(&user),
            tables: tables,
        };

        view.to_html().into_response()
    }
}

mod post {

    use crate::account::Account;
    use crate::csrf;
    use crate::models::csv::tables::Categories;
    use crate::models::csv::types::{CategoriesEntry, ReportEntry, SavingsEntry};
    use crate::models::templates::savings;
    use crate::redirection;
    use crate::storage::{AppendableRepository, BasicRepository};
    use axum::{Extension, Form, response::IntoResponse};
    use axum_csrf::CsrfToken;
    use axum_messages::Messages;

    use tower_sessions::Session;

    pub async fn edit(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        Form(form): Form<savings::EditForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        let mut savings = user.get_savings();

        if form.balance == 0.0 {
            savings.extract(&form.old_subcategory);
        } else {
            // Known performance improvement here
            // - I left it like this to have it more readable (mmyalski)
            savings
                .update(&form.old_subcategory)
                .account_name
                .set(&form.account);
            savings
                .update(&form.old_subcategory)
                .value
                .set(form.balance);
            savings
                .update(&form.old_subcategory)
                .subcategory
                .set(&form.subcategory);
        }

        if let Err(err) = savings.write() {
            messages
                .error("Unable to save changes for savings file.")
                .debug(err.to_string());
            return redirection::to_savings_view().into_response();
        }

        if form.balance == 0.0 {
            messages.success("Savings record deleted.");
        } else {
            messages.success("Savings record updated.");
        }

        redirection::to_savings_view().into_response()
    }

    pub async fn withdraw(
        csrf_token: CsrfToken,
        messages: Messages,
        session: Session,
        Extension(user): Extension<Account>,
        Form(form): Form<savings::WithdrawForm>,
    ) -> impl IntoResponse {
        if !csrf::verify(&session, &csrf_token, &form.csrf_token).await {
            messages.error("CSRF token is invalid.");
            return redirection::to_index().into_response();
        }

        csrf::remove_token(&session).await;

        let mut savings = user.get_savings();

        // 1. Extract the row from database
        let record = match savings.extract(&form.subcategory) {
            Some(s) => s,
            None => {
                messages.error(format!(
                    "Saving for '{}' in '{}' not found.",
                    form.subcategory, form.account
                ));
                return redirection::to_savings_view().into_response();
            }
        };

        let record_value = record.value.get();

        // 2. Modify balance
        match form.amount {
            a if a == 0.0 => {
                messages.info("The amount is 0. No changes have been applied.");
                return redirection::to_savings_view().into_response();
            }
            a if a > record_value => {
                messages.error("The amount to withdraw is too big.");
                return redirection::to_savings_view().into_response();
            }
            a if a == record_value => (),
            a if a < record_value => {
                let savings_entry = SavingsEntry::new()
                    .with_subcategory(record.subcategory.get())
                    .with_account_name(record.account_name.get())
                    .with_value(record_value - form.amount);

                savings.content_mut().push(savings_entry);

                if let Err(err) = savings.write() {
                    messages
                        .error("Cannot update the saving.")
                        .debug(err.to_string());
                    return redirection::to_savings_view().into_response();
                }
            }
            _ => {
                messages.error("Invalid amount to withdraw.");
                return redirection::to_savings_view().into_response();
            }
        };

        // 3. Transfer to report
        let mut report = match user.get_report(form.to_year) {
            Ok(r) => r,
            Err(err) => {
                messages
                    .error("Unable to load selected report.")
                    .debug(err.to_string());
                return redirection::to_savings_view().into_response();
            }
        };

        let report_entry = ReportEntry::new()
            .with_curr_timestamp()
            .with_curr_date()
            .with_subcategory(record.subcategory.get())
            .with_value(-form.amount)
            .with_description("Savings withdraw transfer");

        if let Err(err) = report.append(&report_entry) {
            messages
                .error("Unable to transfer saving to report.")
                .debug(err.to_string());
            return redirection::to_savings_view().into_response();
        }

        let mut categories = match user.get_categories(form.to_year) {
            Ok(c) => c,
            Err(err) => {
                messages
                    .error("Unable to load selected categories.")
                    .debug(err.to_string());
                return redirection::to_savings_view().into_response();
            }
        };

        let category = CategoriesEntry::new()
            .with_category(Categories::SAVINGS_LABEL)
            .with_subcategory(record.subcategory.get());

        if !categories.contains(&category) {
            if let Err(err) = categories.append(&category) {
                messages
                    .error("Unable to create category")
                    .debug(err.to_string());
                return redirection::to_savings_view().into_response();
            }
        }

        messages.info("Saving record updated.");
        redirection::to_savings_view().into_response()
    }
}
