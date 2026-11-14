use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, SpanExporter};
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider,
    Resource,
};
use tracing_subscriber::{prelude::*, EnvFilter};

pub struct Guard {
    tracer: Option<SdkTracerProvider>,
    logger: Option<SdkLoggerProvider>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        if let Some(t) = &self.tracer { let _ = t.shutdown(); }
        if let Some(l) = &self.logger { let _ = l.shutdown(); }
    }
}

pub fn init() -> Guard {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt = tracing_subscriber::fmt::layer();

    // Local dev: no endpoint configured, so only normal console logging.
    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_err() {
        tracing_subscriber::registry().with(filter).with(fmt).init();
        return Guard { tracer: None, logger: None };
    }

    global::set_text_map_propagator(TraceContextPropagator::new());
    let resource = Resource::builder().build(); // reads OTEL_SERVICE_NAME

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(SpanExporter::builder().with_http().build().expect("span exporter"))
        .with_resource(resource.clone())
        .build();
    let logger_provider = SdkLoggerProvider::builder()
        .with_batch_exporter(LogExporter::builder().with_http().build().expect("log exporter"))
        .with_resource(resource)
        .build();
    global::set_tracer_provider(tracer_provider.clone());

    // Keep the exporter's own HTTP chatter out of the exported logs (avoids a feedback loop).
    let otel_logs_filter =
        EnvFilter::new("info,hyper=off,hyper_util=off,h2=off,reqwest=off,opentelemetry=off");

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("nullbox-api")))
        .with(OpenTelemetryTracingBridge::new(&logger_provider).with_filter(otel_logs_filter))
        .init();

    Guard { tracer: Some(tracer_provider), logger: Some(logger_provider) }
}