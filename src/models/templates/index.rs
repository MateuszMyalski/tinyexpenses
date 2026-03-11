use crate::models::templates::{WebPageContext, WebPageToHtml};
use askama::Template;
use chrono::Datelike;

#[derive(Template)]
#[template(path = "index.html")]
pub struct Tmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
}

impl<'a> WebPageToHtml for Tmpl<'a> {}
