use crate::models::templates::{WebPageContext, WebPageToHtml};
use askama::Template;
use chrono::Datelike;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "account_login.html")]
pub struct LoginTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
}

impl<'a> WebPageToHtml for LoginTmpl<'a> {}

#[derive(Template)]
#[template(path = "account_invalid_login.html")]
pub struct InvalidLoginTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
}

impl<'a> WebPageToHtml for InvalidLoginTmpl<'a> {}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "account_view.html")]
pub struct ViewTmpl<'a> {
    pub ctx: &'a WebPageContext<'a>,
    pub api_token: Option<&'a str>,
    pub host_url: &'a str,
}

impl<'a> WebPageToHtml for ViewTmpl<'a> {}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewDetailsForm {
    pub user_full_name: String,
    pub currency: String,
    pub csrf_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewChangePasswordForm {
    pub password: String,
    pub new_password: String,
    pub confirm_password: String,
    pub csrf_token: String,
}

#[derive(Deserialize)]
pub struct ViewApiTokenForm {
    pub csrf_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewChangeColorSchemeForm {
    pub csrf_token: String,
    pub dark_color_scheme: bool,
}
