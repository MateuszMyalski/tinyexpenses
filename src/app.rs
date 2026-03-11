use crate::auth;
use crate::controllers;
use crate::csrf;
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use axum_messages::MessagesManagerLayer;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::OnceLock;
use time::Duration;
use tokio::signal;
use tower_http::services::ServeDir;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing::log::{error, info};

static SECRET_KEY: OnceLock<String> = OnceLock::new();

pub fn get_secret_key() -> &'static str {
    let secret_key = SECRET_KEY.get_or_init(|| {
        env::var("SECRET_KEY").expect("SECRET_KEY environmental variable not found!")
    });

    if secret_key.len() < 32 {
        panic!("Unsafe SECRET_KEY variable is less than 32 characters.");
    }

    secret_key
}

pub struct AppConfig {
    pub bind: SocketAddr,
    pub accounts_dir: PathBuf,
    pub session_expire: time::Duration,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 3000),
            accounts_dir: PathBuf::from("accounts"),
            session_expire: Duration::days(1),
        }
    }
}

pub struct App {
    config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        // Test secret key
        let _ = get_secret_key();

        Self { config }
    }

    pub async fn run(self) {
        info!("Hello from tinyexpenses!");
        info!(
            "Starting backend on directory {:?}",
            self.config.accounts_dir
        );

        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(self.config.session_expire));

        let backend = auth::UsersBackend::new(&self.config.accounts_dir).unwrap_or_else(|err| {
            error!("Unable to start backend: {err}");
            std::process::exit(-1);
        });

        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let app = Router::new()
            .merge(controllers::index::router())
            .nest("/dashboard", controllers::dashboard::router())
            .nest("/savings", controllers::savings::router())
            .nest("/report", controllers::report::router())
            .nest("/categories", controllers::categories::router())
            .nest("/account", controllers::account::router())
            .nest("/plans", controllers::plans::router())
            .nest("/api", controllers::api::router())
            .nest_service("/static", ServeDir::new("static"))
            .layer(MessagesManagerLayer)
            .layer(auth_layer)
            .layer(csrf::create_layer())
            .layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(std::time::Duration::from_secs(10)),
            ));

        info!("Listening on http://{}", self.config.bind);

        let listener = tokio::net::TcpListener::bind(self.config.bind)
            .await
            .unwrap();

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
