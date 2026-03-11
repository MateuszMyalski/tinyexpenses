use crate::app::{App, AppConfig};

mod account;
mod api_token;
mod app;
mod auth;
mod balances;
mod budget;
mod cli;
mod controllers;
mod csrf;
mod extraction;
mod logger;
mod models;
mod redirection;
mod savings;
mod storage;

#[tokio::main]
async fn main() {
    logger::init();

    let args = cli::ServerCli::parse();

    let mut config = AppConfig::default();
    config.bind = args.bind;
    config.accounts_dir = args.accounts_dir;

    let app = App::new(config);
    app.run().await;
}
