//! Types for systemd's [logcontrol] interface.
//!
//! See [`LogControl1`] for the main trait to use (in case you're writing a DBus frontend), or to implement (in case
//! you're integrating a logging framework).
//!
//! [logcontrol]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html

#![deny(warnings, clippy::all, missing_docs, missing_debug_implementations)]
#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter};

use thiserror::Error;

/// A syslog log level as used by the systemd log control interface.
///
/// See [POSIX syslog](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/functions/syslog.html)
/// or `syslog(3)` for more information.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LogLevel {
    /// A panic condition; system is unusable.
    Emerg,
    /// Action must be taken immediately.
    Alert,
    /// A critical condition.
    Crit,
    /// An error.
    Err,
    /// Warnings.
    Warning,
    /// Normal, but significant, condition.
    Notice,
    /// Informational message.
    Info,
    /// Debug-level messages.
    Debug,
}

/// The log level was invalid.
#[derive(Debug, Copy, Clone, Error)]
#[error("Invalid log level")]
pub struct LogLevelParseError;

impl TryFrom<&str> for LogLevel {
    type Error = LogLevelParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "emerg" => Ok(LogLevel::Emerg),
            "alert" => Ok(LogLevel::Alert),
            "crit" => Ok(LogLevel::Crit),
            "err" => Ok(LogLevel::Err),
            "warning" => Ok(LogLevel::Warning),
            "notice" => Ok(LogLevel::Notice),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => Err(LogLevelParseError),
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let level = match self {
            LogLevel::Emerg => "emerg",
            LogLevel::Alert => "alert",
            LogLevel::Crit => "crit",
            LogLevel::Err => "err",
            LogLevel::Warning => "warning",
            LogLevel::Notice => "notice",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
        };
        write!(f, "{level}")
    }
}

/// Log targets used by the systemd log control interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LogTarget {
    /// Log to the console or standard output.
    Console,
    /// The kernel ring message buffer.
    ///
    /// Normally not used by userspace services.
    Kmsg,
    /// Log to the journal natively.
    ///
    /// Prefer this other log targets when running under systemd,
    /// because this log target retains all structured data.
    ///
    /// See [`stderr_connected_to_journal`] to determine whether the current
    /// process is already connected to the journal (i.e. its stderr goes
    /// directly into the systemd journal).
    Journal,
    /// The legacy syslog interface.
    ///
    /// Services which use systemd should prefer the `Journal` interface.
    Syslog,
}

/// The log target was invalid.
#[derive(Debug, Copy, Clone, Error)]
#[error("Invalid log target")]
pub struct LogTargetParseError;

impl TryFrom<&str> for LogTarget {
    type Error = LogTargetParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "console" => Ok(LogTarget::Console),
            "kmsg" => Ok(LogTarget::Kmsg),
            "journal" => Ok(LogTarget::Journal),
            "syslog" => Ok(LogTarget::Syslog),
            _ => Err(LogTargetParseError),
        }
    }
}

impl Display for LogTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let target = match self {
            LogTarget::Console => "console",
            LogTarget::Kmsg => "kmsg",
            LogTarget::Journal => "journal",
            LogTarget::Syslog => "syslog",
        };
        write!(f, "{target}")
    }
}

/// An error in a [`LogControl1`] operation.
#[derive(Debug, Error)]
pub enum LogControl1Error {
    /// A log level is not supported by the underlying log framework.
    #[error("The log level {0} is not supported")]
    UnsupportedLogLevel(LogLevel),
    /// A log target is not supported by the underlying log framework.
    #[error("The log target {0} is not supported")]
    UnsupportedLogTarget(LogTarget),
    /// An IO error occurred while changing log target or log level.
    #[error(transparent)]
    InputOutputError(#[from] std::io::Error),
    /// A generic failure while changing log target or log level.
    #[error("{0}")]
    Failure(String),
}

/// Abstract representation of the [LogControl1] interface.
///
/// Bridges a DBus frontend to a backend logging framework.
///
/// Implementations should choose the initial log target automatically, according
/// to whether their stderr is already connected to the systemd journal directly,
/// per `$JOURNAL_STREAM` (see [`systemd.exec(5)](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)).
/// [`stderr_connected_to_journal`] implements this check.
///
/// [LogControl1]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html
pub trait LogControl1 {
    /// Get the currently configured log level.
    fn level(&self) -> LogLevel;

    /// Set the level of the underlying log framework.
    fn set_level(&mut self, level: LogLevel) -> Result<(), LogControl1Error>;

    /// Get the currently configured log target.
    fn target(&self) -> LogTarget;

    /// Set the target of the underlying log framework.
    fn set_target(&mut self, target: LogTarget) -> Result<(), LogControl1Error>;

    /// Get the syslog identifier.
    fn syslog_identifier(&self) -> &str;
}

/// The DBus object path a log control interface needs to be served on for systemd to find it.
///
/// The path is `/org/freedesktop/LogControl1`, as required by the interface specification.
pub static DBUS_OBJ_PATH: &str = "/org/freedesktop/LogControl1";

/// Whether the current process is directly connected to the systemd journal.
///
/// Use this function to select an appropriate target for [`LogTarget::Auto`].
pub fn stderr_connected_to_journal() -> bool {
    todo!()
}
