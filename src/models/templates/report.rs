use crate::models::csv::types::ReportEntry;
use crate::models::templates::{WebPageContext, WebPageToHtml};
use crate::models::views::{
    BudgetSummaryView, ByMonthDatabasePickerView, ByYearDatabasePickerView, DatabasePickerView,
    MonthReportSummaryView, ValuesAttributes, YearReportSummaryView,
};
use askama::Template;
use chrono::{Datelike, NaiveDate};

#[derive(Template)]
#[template(path = "report_append.html")]
pub struct ViewAppendTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub types_with_categories: Vec<(String, Vec<String>)>,
}

impl<'a> WebPageToHtml for ViewAppendTmpl<'a> {}

#[derive(Template)]
#[template(path = "report_view_year.html")]
pub struct ViewYearTmpl<'a> {
    pub ctx: WebPageContext<'a>,
    pub picker: ByYearDatabasePickerView,
    pub table: YearReportSummaryView,
}

impl<'a> WebPageToHtml for ViewYearTmpl<'a> {}

#[derive(Template)]
#[template(path = "report_view_month.html")]
pub struct ViewMonthTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub picker_year: ByYearDatabasePickerView,
    pub picker_month: ByMonthDatabasePickerView,
    pub view_date: NaiveDate,
    pub table: MonthReportSummaryView,
}

impl<'a> WebPageToHtml for ViewMonthTmpl<'a> {}

#[derive(Template)]
#[template(path = "report_create_report.html")]
pub struct ViewCreateReportTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub year: i32,
}

impl<'a> WebPageToHtml for ViewCreateReportTmpl<'a> {}

#[derive(Template)]
#[template(path = "report_edit.html")]
pub struct EditTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub entries: &'a Vec<ReportEntry>,
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for EditTmpl<'a> {}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateForm {
    pub initial_balance: f32,
    pub csrf_token: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AppendForm {
    pub date: NaiveDate,
    pub value: f32,
    pub description: String,
    pub subcategory: String,
    pub csrf_token: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AppendApi {
    pub subcategory: String,
    pub date: NaiveDate,
    pub value: f32,
    pub description: Option<String>,
}
