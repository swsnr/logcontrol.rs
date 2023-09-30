//! A [`LogControl1`] implementation for [`tracing`].
//!
//! [`TracingLogControl1`] provides a [`LogControl1`] implementation on top of
//! tracing, which uses the [reload layer][tracing_subscriber::reload] to
//! dyanmically switch layers and level filters when the log target or log level
//! are changed over the log control interfaces.
//!
//! It uses a [`LogControl1LayerFactory`] implementation to create the log target
//! layers each time the log target is changed.  This crates provides a default
//! [`PrettyLogControl1LayerFactory`] which uses the pretty format of
//! [`tracing_subscriber`] on stdout for the console target and
//! [`tracing_journald`] for the Journal target.  You can provide your own
//! implementation to customize the layer for each target.
//!
//! When created [`TracingLogControl1`] additionally returns a layer which needs
//! to be added to the global tracing subscriber, i.e. a [`tracing_subscriber::Registry`],
//! for log control to have any effect.
//!
//! ```rust
//! use logcontrol::*;
//! use logcontrol_tracing::*;
//! use tracing_subscriber::prelude::*;
//!
//! let (control, layer) = TracingLogControl1::new(
//!     PrettyLogControl1LayerFactory,
//!     false,
//!     "syslog_identifier".to_string(),
//!     KnownLogTarget::Auto,
//!     LogLevel::Info,
//! ).unwrap();
//!
//! let subscriber = tracing_subscriber::Registry::default().with(layer);
//! tracing::subscriber::set_global_default(subscriber).unwrap();
//! // Then register `control` over DBus, e.g. via `logcontrol_zbus::LogControl1`.
//! ```

#![deny(warnings, clippy::all, missing_docs)]
#![forbid(unsafe_code)]

use logcontrol::{KnownLogTarget, LogControl1, LogControl1Error, LogLevel};
use tracing::Subscriber;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, reload, Layer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TracingLogTarget {
    Console,
    Journal,
    Null,
}

impl From<TracingLogTarget> for KnownLogTarget {
    fn from(value: TracingLogTarget) -> Self {
        match value {
            TracingLogTarget::Console => KnownLogTarget::Console,
            TracingLogTarget::Journal => KnownLogTarget::Journal,
            TracingLogTarget::Null => KnownLogTarget::Null,
        }
    }
}

fn from_known_log_target(
    target: KnownLogTarget,
    connected_to_journal: bool,
) -> Result<TracingLogTarget, LogControl1Error> {
    match target {
        KnownLogTarget::Auto if connected_to_journal => Ok(TracingLogTarget::Journal),
        KnownLogTarget::Auto => Ok(TracingLogTarget::Console),
        KnownLogTarget::Console => Ok(TracingLogTarget::Console),
        KnownLogTarget::Journal => Ok(TracingLogTarget::Journal),
        KnownLogTarget::Null => Ok(TracingLogTarget::Null),
        other => Err(LogControl1Error::UnsupportedLogTarget(
            other.as_str().to_string(),
        )),
    }
}

/// Convert [`logcontrol::LogLevel`] to [`tracing::Level`].
///
/// Return an error if the systemd log level is not supported, i.e. does not map to a
/// corresponding [`tracing::Level`].
pub fn from_log_level(level: LogLevel) -> Result<tracing::Level, LogControl1Error> {
    match level {
        LogLevel::Err => Ok(tracing::Level::ERROR),
        LogLevel::Warning => Ok(tracing::Level::WARN),
        LogLevel::Notice => Ok(tracing::Level::INFO),
        LogLevel::Info => Ok(tracing::Level::DEBUG),
        LogLevel::Debug => Ok(tracing::Level::TRACE),
        unsupported => Err(LogControl1Error::UnsupportedLogLevel(unsupported)),
    }
}

/// Convert [`tracing::Level`] to [`logcontrol::LogLevel`].
fn to_log_level(level: tracing::Level) -> LogLevel {
    match level {
        tracing::Level::ERROR => LogLevel::Err,
        tracing::Level::WARN => LogLevel::Warning,
        tracing::Level::INFO => LogLevel::Notice,
        tracing::Level::DEBUG => LogLevel::Info,
        tracing::Level::TRACE => LogLevel::Debug,
    }
}

/// A factory to create layers for [`TracingLogControl1`].
pub trait LogControl1LayerFactory {
    /// The type of the layer to use for [`KnownLogTarget::Journal`].
    type JournalLayer<S: Subscriber + for<'span> LookupSpan<'span>>: Layer<S>;
    /// The type of the layer to use for [`KnownLogTarget::Console`].
    type ConsoleLayer<S: Subscriber + for<'span> LookupSpan<'span>>: Layer<S>;

    /// Create a layer to use when [`KnownLogTarget::Journal`] is selected.
    ///
    /// The `syslog_identifier` should be send to the journal as `SYSLOG_IDENTIFIER`, to support `journalctl -t`.
    /// See [`systemd.journal-fields(7)`](https://www.freedesktop.org/software/systemd/man/systemd.journal-fields.html).
    fn create_journal_layer<S: Subscriber + for<'span> LookupSpan<'span>>(
        &self,
        syslog_identifier: String,
    ) -> Result<Self::JournalLayer<S>, LogControl1Error>;

    /// Create a layer to use when [`KnownLogTarget::Console`] is selected.
    fn create_console_layer<S: Subscriber + for<'span> LookupSpan<'span>>(
        &self,
    ) -> Result<Self::ConsoleLayer<S>, LogControl1Error>;
}

/// A layer factory which uses pretty printing on stdout for the console target.
///
/// For [`KnownLogTarget::Console`] this layer factory creates a [`mod@tracing_subscriber::fmt`]
/// layer which logs to stdout with the built-in pretty format.
///
/// For [`KnownLogTarget::Journal`] this layer factory creates a [`tracing_journald`]
/// layer without field prefixes and no further customization.
pub struct PrettyLogControl1LayerFactory;

impl LogControl1LayerFactory for PrettyLogControl1LayerFactory {
    type JournalLayer<S: Subscriber + for<'span> LookupSpan<'span>> = tracing_journald::Layer;

    type ConsoleLayer<S: Subscriber + for<'span> LookupSpan<'span>> =
        fmt::Layer<S, fmt::format::Pretty, fmt::format::Format<fmt::format::Pretty>>;

    fn create_journal_layer<S: Subscriber + for<'span> LookupSpan<'span>>(
        &self,
        syslog_identifier: String,
    ) -> Result<Self::JournalLayer<S>, LogControl1Error> {
        Ok(tracing_journald::Layer::new()?
            .with_field_prefix(None)
            .with_syslog_identifier(syslog_identifier))
    }

    fn create_console_layer<S: Subscriber + for<'span> LookupSpan<'span>>(
        &self,
    ) -> Result<Self::ConsoleLayer<S>, LogControl1Error> {
        Ok(tracing_subscriber::fmt::layer().pretty())
    }
}

/// The type of the layer that implements the log target.
pub type LogTargetLayer<F, S> = Layered<
    Option<<F as LogControl1LayerFactory>::ConsoleLayer<S>>,
    Option<<F as LogControl1LayerFactory>::JournalLayer<S>>,
    S,
>;

/// The final type for the layer that implements the log control interface.
pub type LogControl1Layer<F, S> =
    Layered<reload::Layer<LogTargetLayer<F, S>, S>, reload::Layer<LevelFilter, S>, S>;

/// Create a new tracing layer for the given `target`, using the given `factory`.
///
/// We don't handle the `Null` target explicitly here; it disables logging
/// simply because it matches none of the other targets, so we automatically
/// create an empty layer here.
///
/// Return any error returned from the factory methods.
fn make_target_layer<F: LogControl1LayerFactory, S>(
    factory: &F,
    target: TracingLogTarget,
    syslog_identifier: &str,
) -> Result<LogTargetLayer<F, S>, LogControl1Error>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let stdout = if let TracingLogTarget::Console = target {
        Some(factory.create_console_layer::<S>()?)
    } else {
        None
    };
    let journal = if let TracingLogTarget::Journal = target {
        Some(factory.create_journal_layer::<S>(syslog_identifier.to_string())?)
    } else {
        None
    };
    Ok(tracing_subscriber::Layer::and_then(journal, stdout))
}

/// A [`LogControl1`] implementation for [`tracing`].
///
/// This implementation creates a tracing layer which combines two reloadable
/// layers, on for the log target, and another one for the level filter
/// implementing the desired log level.  It keeps the reload handles internally
/// and reloads newly created layers whenever the target or the level is changed.
///
/// Currently, this implementation only supports the following [`KnownLogTarget`]s:
///
/// - [`KnownLogTarget::Console`]
/// - [`KnownLogTarget::Journal`]
/// - [`KnownLogTarget::Null`]
/// - [`KnownLogTarget::Auto`]
///
/// Any other target fails with [`LogControl1Error::UnsupportedLogTarget`].
pub struct TracingLogControl1<F, S>
where
    F: LogControl1LayerFactory,
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    /// Whether the current process is connnected to the systemd journal.
    connected_to_journal: bool,
    /// The syslog identifier used for logging.
    syslog_identifier: String,
    /// The current level active in the level layer.
    level: tracing::Level,
    /// The current target active in the target layer.
    target: TracingLogTarget,
    /// Factory for layers.
    layer_factory: F,
    // /// A handle to reload the level layer in order to change the level.
    level_handle: reload::Handle<LevelFilter, S>,
    // /// A handle to reload the target layer in order to change the target.
    target_handle: reload::Handle<LogTargetLayer<F, S>, S>,
}

impl<F, S> TracingLogControl1<F, S>
where
    F: LogControl1LayerFactory,
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    /// Create a new [`LogControl1`] layer.
    ///
    /// `factory` creates the [`tracing_subscriber::Layer`] for the selected `target`
    /// which denotes the initial log target. The `factory` is invoked whenever the
    /// log target is changed, to create a new layer to use for the selected log
    /// target.  See [`TracingLogControl1`] for supported `target`s.
    ///
    /// `connected_to_journal` indicates whether this process is connected to the systemd
    /// journal. Set to `true` to make [`KnownLogTarget::Auto`] use [`KnownLogTarget::Journal`],
    /// otherwise it uses [`KnownLogTarget::Console`].
    ///
    /// `level` denotes the default tracing log level to start with.
    ///
    /// `syslog_identifier` is passed to [`LogControl1LayerFactory::create_journal_layer`]
    /// for use as `SYSLOG_IDENTIFIER` journal field.
    ///
    /// Returns an error if `target` is not supported, of if creating a layer fails,
    /// e.g. when selecting [`KnownLogTarget::Console`] on a system where journald is
    /// not running, or inside a container which has no direct access to the journald
    /// socket.
    pub fn new(
        factory: F,
        connected_to_journal: bool,
        syslog_identifier: String,
        target: KnownLogTarget,
        level: tracing::Level,
    ) -> Result<(Self, LogControl1Layer<F, S>), LogControl1Error> {
        let tracing_target = from_known_log_target(target, connected_to_journal)?;
        let (target_layer, target_handle) = reload::Layer::new(make_target_layer(
            &factory,
            tracing_target,
            &syslog_identifier,
        )?);
        let (level_layer, level_handle) = reload::Layer::new(LevelFilter::from_level(level));
        let control_layer = Layer::and_then(level_layer, target_layer);
        let control = Self {
            connected_to_journal,
            layer_factory: factory,
            syslog_identifier,
            level,
            target: tracing_target,
            level_handle,
            target_handle,
        };

        Ok((control, control_layer))
    }

    /// Create a new [`LogControl1`] layer with automatic defaults.
    ///
    /// Use [`logcontrol::syslog_identifier()`] as the syslog identifier, and
    /// determine the initial log target automatically according to
    /// [`logcontrol::stderr_connected_to_journal()`].
    ///
    /// `level` denotes the initial level; for `factory` and returned errors,
    ///  see [`Self::new`].
    pub fn new_auto(
        factory: F,
        level: tracing::Level,
    ) -> Result<(Self, LogControl1Layer<F, S>), LogControl1Error> {
        Self::new(
            factory,
            logcontrol::stderr_connected_to_journal(),
            logcontrol::syslog_identifier(),
            KnownLogTarget::Auto,
            level,
        )
    }
}

impl<F, S> LogControl1 for TracingLogControl1<F, S>
where
    F: LogControl1LayerFactory,
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn level(&self) -> LogLevel {
        to_log_level(self.level)
    }

    fn set_level(&mut self, level: LogLevel) -> Result<(), LogControl1Error> {
        let tracing_level = from_log_level(level)?;
        self.level_handle
            .reload(LevelFilter::from_level(tracing_level))
            .map_err(|error| {
                LogControl1Error::Failure(format!(
                    "Failed to reload target layer to switch to log target {level}: {error}"
                ))
            })?;
        self.level = tracing_level;
        Ok(())
    }

    fn target(&self) -> &str {
        KnownLogTarget::from(self.target).as_str()
    }

    fn set_target<T: AsRef<str>>(&mut self, target: T) -> Result<(), LogControl1Error> {
        let new_tracing_target = from_known_log_target(
            KnownLogTarget::try_from(target.as_ref())?,
            self.connected_to_journal,
        )?;
        let new_layer = make_target_layer(
            &self.layer_factory,
            new_tracing_target,
            &self.syslog_identifier,
        )?;
        self.target_handle.reload(new_layer).map_err(|error| {
            LogControl1Error::Failure(format!(
                "Failed to reload target layer to switch to log target {}: {error}",
                target.as_ref()
            ))
        })?;
        self.target = new_tracing_target;
        Ok(())
    }

    fn syslog_identifier(&self) -> &str {
        &self.syslog_identifier
    }
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_impl_all;
    use tracing_subscriber::Registry;

    use crate::{PrettyLogControl1LayerFactory, TracingLogControl1};

    // Ensure that the our default log control layers are Send and Sync, this is required for zbus.
    assert_impl_all!(TracingLogControl1<PrettyLogControl1LayerFactory, Registry>: Send, Sync);
}
