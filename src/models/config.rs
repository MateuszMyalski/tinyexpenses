use crate::api_token;
use crate::storage;
use scrypt::{
    Scrypt,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::error::Error;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Tinyexpenses {
    pub currency: String,
    pub api_token: String,
    pub dark_color_scheme: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct User {
    pub full_name: String,
    pub username: String,
    pub password_hash: String,
    pub active: bool,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ConfigContent {
    pub user: User,
    pub tinyexpenses: Tinyexpenses,
}

pub type Config = storage::TomlFile<ConfigContent>;

impl Config {
    pub fn user_full_name(&self) -> &str {
        &self.content().user.full_name
    }

    pub fn set_user_full_name(&mut self, full_name: &str) {
        self.content_mut().user.full_name = full_name.to_string();
    }

    pub fn user_username(&self) -> &str {
        &self.content().user.username
    }

    #[allow(dead_code)]
    // TODO(mmyalski) Introduce user managing
    pub fn set_user_username(&mut self, username: &str) {
        self.content_mut().user.username = username.to_string();
    }

    pub fn user_password_hash(&self) -> &str {
        &self.content().user.password_hash
    }

    pub fn set_user_password_hash(&mut self, password: &str) -> Result<(), Box<dyn Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Scrypt
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        self.content_mut().user.password_hash = hash;

        Ok(())
    }

    pub fn user_empty_password(&self) -> bool {
        self.content().user.password_hash.is_empty()
    }

    pub fn user_password_verify(&self, password: &str) -> bool {
        PasswordHash::new(&self.content().user.password_hash)
            .ok()
            .and_then(|hash| Scrypt.verify_password(password.as_bytes(), &hash).ok())
            .is_some()
    }

    pub fn user_active(&self) -> bool {
        self.content().user.active
    }

    #[allow(dead_code)]
    // TODO(mmyalski) Introduce user managing
    pub fn set_user_active(&mut self, user_active: bool) {
        self.content_mut().user.active = user_active;
    }

    pub fn currency(&self) -> &str {
        &self.content().tinyexpenses.currency
    }

    pub fn set_currency(&mut self, currency: impl Into<String>) {
        self.content_mut().tinyexpenses.currency = currency.into();
    }

    pub fn api_token(&self) -> &str {
        &self.content().tinyexpenses.api_token
    }

    pub fn generate_token(&mut self) -> Result<(), Box<dyn Error>> {
        self.content_mut().tinyexpenses.api_token = api_token::generate(32);
        Ok(())
    }

    pub fn set_dark_color_scheme(&mut self) {
        self.content_mut().tinyexpenses.dark_color_scheme = true
    }

    pub fn set_light_color_scheme(&mut self) {
        self.content_mut().tinyexpenses.dark_color_scheme = false
    }

    pub fn dark_color_scheme(&self) -> bool {
        self.content().tinyexpenses.dark_color_scheme
    }
}
