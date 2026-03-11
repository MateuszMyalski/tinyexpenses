use crate::account::Account;
use crate::csrf;
use crate::models::config::Config;
use axum_csrf::CsrfToken;
use axum_messages::Messages;
use chrono::{DateTime, Local};
use tower_sessions::Session;
use tracing::log::error;

pub mod account;
pub mod categories;
pub mod dashboard;
pub mod error;
pub mod index;
pub mod plans;
pub mod report;
pub mod savings;

pub trait WebPageToHtml {
    fn to_html(&self) -> axum::response::Html<String>
    where
        Self: askama::Template,
    {
        let html = self.render().unwrap();
        axum::response::Html(html)
    }
}

pub struct WebPageContext<'a> {
    title: &'a str,
    now: DateTime<Local>,
    messages: Vec<(String, String)>,
    account: Option<&'a Config>,
    csrf_token: Option<String>,
}

impl<'a> WebPageContext<'a> {
    pub fn new(title: &'a str) -> Self {
        WebPageContext {
            title: title,
            now: Local::now(),
            messages: vec![],
            account: None,
            csrf_token: None,
        }
    }

    pub fn now(&self) -> &DateTime<Local> {
        &self.now
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn with_messages(mut self, messages: Messages) -> Self {
        self.messages = messages
            .into_iter()
            .map(|m| (m.level.to_string(), m.to_string()))
            .collect();

        self
    }

    pub fn messages(&self) -> &Vec<(String, String)> {
        &self.messages
    }

    pub async fn with_csrf(mut self, csrf_token: &'a CsrfToken, session: &'a Session) -> Self {
        self.csrf_token = Some(csrf::regenerate_token(session, csrf_token).await);
        self
    }

    pub fn csrf_token(&self) -> &String {
        match &self.csrf_token {
            Some(c) => &c,
            None => {
                error!("Tried to use uninitialized CSRF token.");
                panic!("Tried to use uninitialized CSRF token.");
            }
        }
    }

    pub fn with_account(mut self, account: &'a Account) -> Self {
        self.account = Some(&account.config);
        self
    }

    pub fn account(&self) -> &Config {
        match &self.account {
            Some(a) => &a,
            None => {
                error!("Tried to use uninitialized CSRF token.");
                panic!("Tried to use uninitialized account config.");
            }
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CommonCsvEditForm {
    pub csv_content: String,
    pub csrf_token: String,
}

#[derive(serde::Deserialize)]
pub struct WithTemplateCreateForm {
    pub template_year: i32,
    pub csrf_token: String,
}
