use crate::models::resources;
use crate::models::templates::{WebPageContext, WebPageToHtml};
use crate::models::views::{ByYearDatabasePickerView, DatabasePickerView, SavingsAccountView};
use askama::Template;
use chrono::Datelike;

#[derive(Template)]
#[template(path = "savings_view.html")]
pub struct ViewTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub tables: Vec<resources::SavingsAccount>,
}

impl<'a> WebPageToHtml for ViewTmpl<'a> {}

#[derive(Template)]
#[template(path = "savings_edit.html")]
pub struct EditTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub subcategory: &'a str,
    pub account: &'a str,
    pub balance: f32,
}

impl<'a> WebPageToHtml for EditTmpl<'a> {}

#[derive(serde::Deserialize)]
pub struct ViewForm {
    pub subcategory: String,
    pub account: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct EditForm {
    pub old_subcategory: String,
    pub subcategory: String,
    pub account: String,
    pub balance: f32,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "savings_withdraw.html")]
pub struct WithdrawTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub subcategory: String,
    pub account: String,
    pub balance: f32,
    pub picker: ByYearDatabasePickerView,
}

impl<'a> WebPageToHtml for WithdrawTmpl<'a> {}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct WithdrawForm {
    pub subcategory: String,
    pub account: String,
    pub amount: f32,
    pub to_year: i32,
    pub csrf_token: String,
}
