use nullbox_domain::DomainEvent;

/// Where domain events go. The MVP only logs them; audit logs, email,
/// webhooks and metrics are more implementations of this trait (or a fan-out).
pub trait EventSink: Send + Sync {
    fn emit(&self, event: DomainEvent);
}

pub struct TracingEventSink;

impl EventSink for TracingEventSink {
    fn emit(&self, event: DomainEvent) {
        let payload = serde_json::to_string(&event).unwrap_or_default();
        tracing::info!(target: "nullbox::events", %payload, "domain event");
    }
}
