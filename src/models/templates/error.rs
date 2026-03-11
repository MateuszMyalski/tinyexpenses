use crate::models::templates::{WebPageContext, WebPageToHtml};
use askama::Template;

#[derive(Template)]
#[template(path = "error_url.html")]
pub struct UrlTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
}

impl<'a> WebPageToHtml for UrlTmpl<'a> {}
