use crate::models::templates::{WebPageContext, WebPageToHtml};
use crate::models::views::{
    ByYearDatabasePickerView, DatabasePickerView, GraphView, YearGraphView,
};
use askama::Template;
use chrono::Datelike;

#[derive(Template)]
#[template(path = "graphs_year_details.html")]
pub struct GraphsYearTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub graph: YearGraphView,
    pub months_label: [&'a str; 12],
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for GraphsYearTmpl<'a> {}
