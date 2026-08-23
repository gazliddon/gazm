use crate::opts::Opts;
use colored::{ColoredString, Colorize};
use serde::Deserialize;
use std::cell::RefCell;
use std::fmt as std_fmt;
use std::fmt::Write as _;
use std::time::Instant;
use tracing::{field::Visit, Event, Span, Subscriber};
use tracing_subscriber::{
    fmt::{
        self as subscriber_fmt,
        format::{FormatEvent, Writer},
        FmtContext,
    },
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

thread_local! {
    static CAPTURED_EVENTS: RefCell<Option<String>> = const { RefCell::new(None) };
    static CAPTURED_TIMING: RefCell<Option<TimingReport>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone, Default)]
pub struct TimingReport {
    pub stages: Vec<(String, std::time::Duration)>,
    pub source_files: usize,
    pub source_lines: usize,
    pub token_cache_hits: usize,
    pub token_cache_misses: usize,
}

#[cfg(feature = "chrome-trace")]
pub type TraceGuard = tracing_chrome::FlushGuard;
#[cfg(not(feature = "chrome-trace"))]
pub type TraceGuard = ();

/// Capture tracing output on the current thread and return it as one block.
/// This lets independent workspace targets build concurrently without
/// interleaving their human-readable progress output.
pub fn capture<R>(f: impl FnOnce() -> R) -> (R, String, TimingReport) {
    CAPTURED_EVENTS.with(|captured| {
        *captured.borrow_mut() = Some(String::new());
        CAPTURED_TIMING.with(|timing| {
            *timing.borrow_mut() = Some(TimingReport::default());
            let result = f();
            let output = captured.borrow_mut().take().unwrap_or_default();
            let report = timing.borrow_mut().take().unwrap_or_default();
            (result, output, report)
        })
    })
}

pub fn record_parse_stats(source_files: usize, source_lines: usize) {
    CAPTURED_TIMING.with(|timing| {
        if let Some(report) = timing.borrow_mut().as_mut() {
            report.source_files = source_files;
            report.source_lines = source_lines;
        }
    });
}

pub fn record_token_cache_hit() {
    CAPTURED_TIMING.with(|timing| {
        if let Some(report) = timing.borrow_mut().as_mut() {
            report.token_cache_hits += 1;
        }
    });
}

pub fn record_token_cache_miss() {
    CAPTURED_TIMING.with(|timing| {
        if let Some(report) = timing.borrow_mut().as_mut() {
            report.token_cache_misses += 1;
        }
    });
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize)]
pub enum Verbosity {
    #[default]
    Silent = 0,
    Normal = 1,
    Info = 2,
    Interesting = 3,
    Debug = 4,
}

pub trait Messageize {
    fn error(self) -> ColoredString;
    fn info(self) -> ColoredString;
    fn success(self) -> ColoredString;
}

impl Messageize for &str {
    fn error(self) -> ColoredString {
        self.red().bold()
    }
    fn info(self) -> ColoredString {
        self.blue().bold()
    }
    fn success(self) -> ColoredString {
        self.green().bold()
    }
}

impl Messageize for String {
    fn error(self) -> ColoredString {
        self.red().bold()
    }
    fn info(self) -> ColoredString {
        self.blue().bold()
    }
    fn success(self) -> ColoredString {
        self.green().bold()
    }
}

pub fn init(opts: &Opts, trace_file: Option<&std::path::Path>) -> Option<TraceGuard> {
    let level = match opts.verbose {
        Verbosity::Silent => "warn",
        Verbosity::Normal => "info",
        Verbosity::Info => "debug",
        Verbosity::Interesting | Verbosity::Debug => "trace",
    };
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("gazm={level}")));
    let registry = tracing_subscriber::registry().with(filter).with(
        subscriber_fmt::layer()
            .with_target(false)
            .without_time()
            .event_format(IndentedFormat),
    );

    #[cfg(feature = "chrome-trace")]
    if let Some(path) = trace_file {
        let (layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
            .file(path)
            .include_args(true)
            .build();
        // The guard flushes the trace when dropped. Keep it alive until exit.
        let _ = registry.with(layer).try_init();
        return Some(guard);
    }

    #[cfg(not(feature = "chrome-trace"))]
    if trace_file.is_some() {
        eprintln!(" WARN --trace requires Gazm built with --features chrome-trace");
    }

    let _ = registry.try_init();
    None
}

/// Compact CLI formatter that keeps nested status/operation messages readable
/// without adding timestamps to short-lived assembler output.
struct IndentedFormat;

impl<S, N> FormatEvent<S, N> for IndentedFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> subscriber_fmt::format::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std_fmt::Result {
        let depth = ctx.event_scope().map_or(0, |scope| scope.count());
        let indent = "   ".repeat(depth);
        let mut fields = EventFields::default();
        event.record(&mut fields);
        let level = match *event.metadata().level() {
            tracing::Level::ERROR => "ERROR".red().bold(),
            tracing::Level::WARN => " WARN".yellow().bold(),
            tracing::Level::INFO => " INFO".green().bold(),
            tracing::Level::DEBUG => "DEBUG".blue().bold(),
            tracing::Level::TRACE => "TRACE".purple().bold(),
        };
        let line = format!("{indent}{level} {}\n", fields.value);
        CAPTURED_EVENTS.with(|captured| {
            if let Some(output) = captured.borrow_mut().as_mut() {
                output.push_str(&line);
                Ok(())
            } else {
                writer.write_str(&line)
            }
        })
    }
}

#[derive(Default)]
struct EventFields {
    value: String,
}

impl Visit for EventFields {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.value.push_str(value);
        } else {
            self.add_field(field.name(), &value);
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let formatted = format!("{value:?}");
        if field.name() == "message" {
            self.value = formatted.trim_matches('"').to_owned();
        } else {
            self.add_field(field.name(), &formatted);
        }
    }
}

impl EventFields {
    fn add_field(&mut self, name: &str, value: &dyn std::fmt::Display) {
        if !self.value.is_empty() {
            self.value.push(' ');
        }
        self.value.push_str(name);
        self.value.push('=');
        self.value.push_str(&value.to_string());
    }
}

pub fn status<F, Y, S>(text: S, mut f: F) -> Y
where
    F: FnMut(()) -> Y,
    S: Into<String>,
{
    let text = text.into();
    let started = Instant::now();
    let span = tracing::info_span!("status", message = %text);
    tracing::info!("{text}");
    let _entered = span.enter();
    let result = f(());
    CAPTURED_TIMING.with(|timing| {
        if let Some(report) = timing.borrow_mut().as_mut() {
            report.stages.push((text.clone(), started.elapsed()));
        }
    });
    result
}

/// Format short stage timings consistently in the CLI output.
pub fn format_duration(duration: std::time::Duration) -> String {
    let micros = duration.as_secs_f64() * 1_000_000.0;
    if micros < 1_000.0 {
        format!("{micros:.0} µs")
    } else if micros < 1_000_000.0 {
        format!("{:.2} ms", micros / 1_000.0)
    } else {
        format!("{:.2} s", micros / 1_000_000.0)
    }
}

/// Format an integer with thousands separators for human-readable reports.
pub fn format_count(value: usize) -> String {
    let digits = value.to_string();
    let first = digits.len() % 3;
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);

    if first != 0 {
        formatted.push_str(&digits[..first]);
    }
    for chunk in digits.as_bytes()[first..].chunks(3) {
        if !formatted.is_empty() {
            formatted.push(',');
        }
        formatted.push_str(std::str::from_utf8(chunk).expect("digits are valid UTF-8"));
    }
    formatted
}

pub fn info<F, Y, S>(text: S, mut f: F) -> Y
where
    F: FnMut(()) -> Y,
    S: Into<String>,
{
    let text = text.into();
    let span = tracing::debug_span!("operation", message = %text);
    tracing::debug!("{text}");
    let _entered = span.enter();
    f(())
}

#[allow(dead_code)]
pub fn debug<F, Y>(text: &str, mut f: F) -> Y
where
    F: FnMut(()) -> Y,
{
    let span = tracing::trace_span!("debug", message = %text);
    tracing::trace!("{text}");
    let _entered = span.enter();
    f(())
}

pub fn status_message(args: std_fmt::Arguments<'_>) {
    tracing::info!("{}", args);
}
pub fn info_message(args: std_fmt::Arguments<'_>) {
    tracing::debug!("{}", args);
}
pub fn interesting_message(args: std_fmt::Arguments<'_>) {
    tracing::trace!("{}", args);
}
pub fn debug_message(args: std_fmt::Arguments<'_>) {
    tracing::trace!("{}", args);
}
pub fn error_message(args: std_fmt::Arguments<'_>) {
    tracing::error!("{}", args);
}

#[macro_export]
macro_rules! status_mess {
    ($($arg:tt)*) => { $crate::messages::status_message(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! info_mess {
    ($($arg:tt)*) => { $crate::messages::info_message(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! interesting_mess {
    ($($arg:tt)*) => { $crate::messages::interesting_message(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! debug_mess {
    ($($arg:tt)*) => { $crate::messages::debug_message(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! status_err {
    ($($arg:tt)*) => { $crate::messages::error_message(format_args!($($arg)*)) };
}

#[allow(dead_code)]
pub fn span() -> Span {
    tracing::info_span!("operation")
}
