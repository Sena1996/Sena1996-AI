use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub struct LogConfig {
    level: Level,
    json: bool,
    include_span_events: bool,
}

impl LogConfig {
    pub fn new() -> Self {
        Self {
            level: Level::INFO,
            json: false,
            include_span_events: false,
        }
    }

    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn with_level_str(mut self, level: &str) -> Self {
        self.level = match level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        };
        self
    }

    pub fn json(mut self, json: bool) -> Self {
        self.json = json;
        self
    }

    pub fn with_span_events(mut self, include: bool) -> Self {
        self.include_span_events = include;
        self
    }

    pub fn init(self) {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(self.level.as_str()));

        let span_events = if self.include_span_events {
            FmtSpan::NEW | FmtSpan::CLOSE
        } else {
            FmtSpan::NONE
        };

        if self.json {
            let fmt_layer = fmt::layer()
                .json()
                .with_span_events(span_events)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        } else {
            let fmt_layer = fmt::layer()
                .with_span_events(span_events)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .init();
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn init_logging(config: &sena_core::config::ObserveConfig) {
    LogConfig::new()
        .with_level_str(&config.log_level)
        .json(false)
        .init();
}

#[macro_export]
macro_rules! log_request {
    ($provider:expr, $status:expr, $duration:expr) => {
        tracing::info!(
            provider = $provider,
            status = $status,
            duration_ms = $duration,
            "request completed"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($provider:expr, $error:expr) => {
        tracing::error!(
            provider = $provider,
            error = %$error,
            "request failed"
        );
    };
}

#[macro_export]
macro_rules! log_circuit_state {
    ($provider:expr, $old:expr, $new:expr) => {
        tracing::warn!(
            provider = $provider,
            old_state = ?$old,
            new_state = ?$new,
            "circuit breaker state changed"
        );
    };
}
