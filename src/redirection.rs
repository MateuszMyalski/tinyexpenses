use crate::budget::BudgetError;
use axum::response::Redirect;
use std::error::Error;
use tracing::log::debug;

pub fn by_error(err: Box<dyn Error>, year: i32) -> Redirect {
    if let Some(budget_err) = err.downcast_ref::<BudgetError>() {
        match budget_err {
            BudgetError::Categories(_) => {
                return to_categories_create(year);
            }
            BudgetError::Report(_) => {
                return to_report_create(year);
            }
            BudgetError::Plans(_) => {
                return to_plans_create(year);
            }
        }
    } else {
        debug!("{}", err);
        panic!("Unhandled error!");
    }
}

pub fn to_index() -> Redirect {
    Redirect::to("/")
}

pub fn to_dashboard() -> Redirect {
    Redirect::to("/dashboard")
}

pub fn to_account_view() -> Redirect {
    Redirect::to("/account/view")
}

pub fn to_plans_view(year: i32) -> Redirect {
    Redirect::to(&format!("/plans/view/{year}"))
}

pub fn to_plans_create(year: i32) -> Redirect {
    Redirect::to(&format!("/plans/create/{year}"))
}

pub fn to_plans_edit(year: i32) -> Redirect {
    Redirect::to(&format!("/plans/edit/{year}"))
}

pub fn to_categories_create(year: i32) -> Redirect {
    Redirect::to(&format!("/categories/create/{year}"))
}

pub fn to_categories_edit(year: i32) -> Redirect {
    Redirect::to(&format!("/categories/edit/{year}"))
}

pub fn to_savings_view() -> Redirect {
    Redirect::to("/savings/view")
}

pub fn to_report_edit(year: i32) -> Redirect {
    Redirect::to(&format!("/report/edit/{year}"))
}

pub fn to_report_create(year: i32) -> Redirect {
    Redirect::to(&format!("/report/create/{year}"))
}

pub fn to_report_view(year: i32) -> Redirect {
    Redirect::to(&format!("/report/view/{year}"))
}

pub fn to_report_append(year: i32) -> Redirect {
    Redirect::to(&format!("/report/append/{year}"))
}
