use tracing_subscriber::{
    EnvFilter, fmt, fmt::format::FormatEvent, fmt::format::Writer, layer::SubscriberExt,
    registry::LookupSpan, util::SubscriberInitExt,
};

struct ComponentFormat;

impl<S, N> FormatEvent<S, N> for ComponentFormat
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> fmt::format::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // Extract component and message fields
        let mut component = None;
        let mut message = None;
        let mut extractor = FieldExtractor {
            component: &mut component,
            message: &mut message,
        };
        event.record(&mut extractor);

        // Write level
        let level = *event.metadata().level();
        write!(writer, "{:5} ", level)?;

        // Write component prefix if present
        if let Some(comp) = component.as_deref() {
            write!(writer, "[{}] ", comp)?;
        }

        // Write message text if present
        if let Some(msg) = message.as_deref() {
            write!(writer, "{} ", msg)?;
        }

        // Write the rest of the event using a custom field formatter that excludes component and message
        let mut field_visitor = FilteredFieldVisitor {
            writer: writer.by_ref(),
        };
        event.record(&mut field_visitor);
        writeln!(writer)
    }
}

struct FilteredFieldVisitor<'a> {
    writer: Writer<'a>,
}

impl<'a> tracing::field::Visit for FilteredFieldVisitor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let name = field.name();
        if name != "component" && name != "message" {
            let _ = write!(self.writer, "{}={} ", name, value);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        if name != "component" && name != "message" {
            let _ = write!(self.writer, "{}={:?} ", name, value);
        }
    }
}

struct FieldExtractor<'a> {
    component: &'a mut Option<String>,
    message: &'a mut Option<String>,
}

impl<'a> tracing::field::Visit for FieldExtractor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "component" => *self.component = Some(value.to_string()),
            "message" => *self.message = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "component" => *self.component = Some(format!("{:?}", value)),
            "message" => *self.message = Some(format!("{:?}", value)),
            _ => {}
        }
    }
}

/// Initialize the tracing subscriber with custom formatter.
/// Component field will be displayed as [component] prefix.
/// Defaults to INFO level if RUST_LOG is not set.
/// Excludes SQL query logs from sea-orm/sqlx as they are too verbose.
pub fn init() {
    let base_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Always filter out verbose SQL logs by setting sqlx to warn level
    // This suppresses the long query logs while still showing errors
    let filter = base_filter
        .add_directive("sqlx=warn".parse().unwrap())
        .add_directive("sea_orm=warn".parse().unwrap());

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(false)
                .with_ansi(true)
                .with_writer(std::io::stderr)
                .event_format(ComponentFormat),
        )
        .with(filter)
        .init();
}
