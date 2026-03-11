use crate::models::csv::types::PlansEntry;
use crate::models::templates::{WebPageContext, WebPageToHtml};
use crate::models::views::{
    BudgetSummaryView, ByYearDatabasePickerView, DatabasePickerView, PlansSummaryView,
};
use askama::Template;
use chrono::{Datelike, NaiveDate};

#[derive(Template)]
#[template(path = "plans_create.html")]
pub struct CreateTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub requested_year: i32,
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for CreateTmpl<'a> {}

#[derive(Template)]
#[template(path = "plans_view.html")]
pub struct ViewTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub picker: ByYearDatabasePickerView,
    pub view_date: NaiveDate,
    pub table: PlansSummaryView,
}

impl<'a> WebPageToHtml for ViewTmpl<'a> {}

#[derive(Template)]
#[template(path = "plans_edit.html")]
pub struct EditTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub entries: &'a Vec<PlansEntry>,
    pub months_labels: [&'static str; 12],
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for EditTmpl<'a> {}
