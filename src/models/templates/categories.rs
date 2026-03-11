use crate::models::csv::types::CategoriesEntry;
use crate::models::templates::{WebPageContext, WebPageToHtml};
use crate::models::views::{ByYearDatabasePickerView, DatabasePickerView};
use askama::Template;
use chrono::Datelike;

#[derive(Template)]
#[template(path = "categories_create.html")]
pub struct CreateTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub requested_year: i32,
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for CreateTmpl<'a> {}

#[derive(Template)]
#[template(path = "categories_edit.html")]
pub struct EditTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub categories: &'a Vec<CategoriesEntry>,
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for EditTmpl<'a> {}
