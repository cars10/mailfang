use tracing_subscriber::{
    fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, 
    fmt::format::FormatEvent, fmt::format::Writer, registry::LookupSpan
};

struct ComponentFormat;

impl<S, N> FormatEvent<S, N> for ComponentFormat
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> fmt::format::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // Extract component field
        let mut component = None;
        let mut visitor = ComponentVisitor { component: &mut component };
        event.record(&mut visitor);
        
        // Write level
        let level = *event.metadata().level();
        write!(writer, "{:5} ", level)?;
        
        // Write component prefix if present
        if let Some(comp) = component {
            write!(writer, "[{}] ", comp)?;
        }
        
        // Write the rest of the event using the field formatter
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

struct ComponentVisitor<'a> {
    component: &'a mut Option<String>,
}

impl<'a> tracing::field::Visit for ComponentVisitor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "component" {
            *self.component = Some(value.to_string());
        }
    }
    
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "component" {
            *self.component = Some(format!("{:?}", value));
        }
    }
}

/// Initialize the tracing subscriber with custom formatter.
/// Component field will be displayed as [component] prefix.
/// Defaults to INFO level if RUST_LOG is not set.
/// Excludes SQL query logs from sea-orm/sqlx as they are too verbose.
pub fn init() {
    let base_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
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
                .event_format(ComponentFormat)
        )
        .with(filter)
        .init();
}
