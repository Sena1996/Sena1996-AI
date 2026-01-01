use once_cell::sync::Lazy;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::time::Instant;

static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

static REQUEST_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new("sena_requests_total", "Total number of requests");
    let counter = CounterVec::new(opts, &["provider", "status"]).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    let opts = HistogramOpts::new("sena_request_duration_seconds", "Request duration in seconds")
        .buckets(vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]);
    let histogram = HistogramVec::new(opts, &["provider"]).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static TOKEN_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    let opts = Opts::new("sena_tokens_total", "Total tokens used");
    let counter = CounterVec::new(opts, &["provider", "type"]).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static CACHE_HITS: Lazy<Counter> = Lazy::new(|| {
    let opts = Opts::new("sena_cache_hits_total", "Total cache hits");
    let counter = Counter::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static CACHE_MISSES: Lazy<Counter> = Lazy::new(|| {
    let opts = Opts::new("sena_cache_misses_total", "Total cache misses");
    let counter = Counter::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(counter.clone())).unwrap();
    counter
});

static CIRCUIT_STATE: Lazy<GaugeVec> = Lazy::new(|| {
    let opts = Opts::new("sena_circuit_state", "Circuit breaker state (0=closed, 1=open, 2=half-open)");
    let gauge = GaugeVec::new(opts, &["provider"]).unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

static ACTIVE_SESSIONS: Lazy<Gauge> = Lazy::new(|| {
    let opts = Opts::new("sena_active_sessions", "Number of active sessions");
    let gauge = Gauge::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(gauge.clone())).unwrap();
    gauge
});

static EMBEDDING_DURATION: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new("sena_embedding_duration_seconds", "Embedding generation duration")
        .buckets(vec![0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]);
    let histogram = Histogram::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

static VECTOR_SEARCH_DURATION: Lazy<Histogram> = Lazy::new(|| {
    let opts = HistogramOpts::new("sena_vector_search_duration_seconds", "Vector search duration")
        .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25]);
    let histogram = Histogram::with_opts(opts).unwrap();
    REGISTRY.register(Box::new(histogram.clone())).unwrap();
    histogram
});

pub struct Metrics;

impl Metrics {
    pub fn record_request(provider: &str, status: &str) {
        REQUEST_COUNTER.with_label_values(&[provider, status]).inc();
    }

    pub fn record_request_duration(provider: &str, duration_secs: f64) {
        REQUEST_DURATION
            .with_label_values(&[provider])
            .observe(duration_secs);
    }

    pub fn record_tokens(provider: &str, prompt_tokens: u32, completion_tokens: u32) {
        TOKEN_COUNTER
            .with_label_values(&[provider, "prompt"])
            .inc_by(prompt_tokens as f64);
        TOKEN_COUNTER
            .with_label_values(&[provider, "completion"])
            .inc_by(completion_tokens as f64);
    }

    pub fn record_cache_hit() {
        CACHE_HITS.inc();
    }

    pub fn record_cache_miss() {
        CACHE_MISSES.inc();
    }

    pub fn set_circuit_state(provider: &str, state: CircuitStateMetric) {
        CIRCUIT_STATE
            .with_label_values(&[provider])
            .set(state as i64 as f64);
    }

    pub fn set_active_sessions(count: i64) {
        ACTIVE_SESSIONS.set(count as f64);
    }

    pub fn record_embedding_duration(duration_secs: f64) {
        EMBEDDING_DURATION.observe(duration_secs);
    }

    pub fn record_vector_search_duration(duration_secs: f64) {
        VECTOR_SEARCH_DURATION.observe(duration_secs);
    }

    pub fn gather() -> String {
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let metric_families = REGISTRY.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i64)]
pub enum CircuitStateMetric {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

impl From<sena_core::CircuitState> for CircuitStateMetric {
    fn from(state: sena_core::CircuitState) -> Self {
        match state {
            sena_core::CircuitState::Closed => Self::Closed,
            sena_core::CircuitState::Open => Self::Open,
            sena_core::CircuitState::HalfOpen => Self::HalfOpen,
        }
    }
}

pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    pub fn record_request(self, provider: &str) {
        Metrics::record_request_duration(provider, self.elapsed_secs());
    }

    pub fn record_embedding(self) {
        Metrics::record_embedding_duration(self.elapsed_secs());
    }

    pub fn record_vector_search(self) {
        Metrics::record_vector_search_duration(self.elapsed_secs());
    }
}
