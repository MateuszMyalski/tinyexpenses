use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "tinyexpenses-server")]
#[command(about = "TinyExpenses backend server")]
pub struct ServerCli {
    /// Address the HTTP server will bind to
    #[arg(long, default_value = "0.0.0.0:3000")]
    pub bind: SocketAddr,

    /// Path to accounts directory
    #[arg(long = "db", default_value = "accounts")]
    pub accounts_dir: PathBuf,
}

impl ServerCli {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
