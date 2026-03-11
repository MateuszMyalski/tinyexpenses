use std::panic;
use std::sync::OnceLock;
use tracing::log::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn panic_hook(panic_info: &panic::PanicHookInfo<'_>) {
    if let Some(location) = panic_info.location() {
        error!(
            "Panic occurred in file '{}' at line {}",
            location.file(),
            location.line(),
        );
    } else {
        error!("Panic occurred but can't get location information...");
    }

    if let Some(payload) = panic_info.payload_as_str() {
        error!("{}", payload);
    } else {
        error!("Cannot read the payload from panic.");
    }
}

pub fn init() {
    let filter = EnvFilter::new("warn,tinyexpenses=trace,tokio=warn,axum=warn");

    let file_appender = rolling::daily("logs", "tinyexpenses.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let stdout_layer = fmt::layer();
    let file_layer = fmt::layer().with_writer(file_writer).with_ansi(false); // no colors in file

    LOG_GUARD.set(guard).expect("logger already initialized");

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    panic::set_hook(Box::new(panic_hook));
}
