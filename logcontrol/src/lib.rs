//! Types for systemd's [logcontrol] interface.
//!
//! ## The log control interface
//!
//! The log control interface exposes the basic log settings of a service over a
//! specified DBus interface under a fixed DBus object path.  If a systemd
//! service then defines a fixed DBus name in its unit file, via the `BusName`
//! property in the `Service` section, `systemctl` can query and update the
//! logging settings over DBus.
//!
//! For instance, `systemd-resolved.service` specifies a bus name in its unit
//! file:
//!
//! ```ini
//! BusName=org.freedesktop.resolve1
//! ```
//!
//! It also exports the log control interface:
//!
//! ```console
//! $ busctl tree org.freedesktop.resolve1
//! └─ /org
//!   └─ /org/freedesktop
//!     ├─ /org/freedesktop/LogControl1
//!     […]
//! ```
//!
//! Hence, we can use `systemctl` to query the log level of the service and
//! change it at runtime, e.g. to enable verbose debugging logging for the
//! running service instance:
//!
//! ```console
//! # systemctl service-log-level systemd-resolved.service
//! info
//! # systemctl service-log-level systemd-resolved.service debug
//! # systemctl service-log-level systemd-resolved.service
//! debug
//! ```
//!
//! This crate provides abstract types to implement and expose this interface.
//!
//! ## Provided types and utilities
//!
//! The [`LogControl1`] trait implements abstract log control interface in Rust.
//! To add support for a logging framework you need to implement this trait
//! around a `struct` which can dynamically adapt the logging output as well as
//! the logging level.
//!
//! To expose an implementation of the log control interface use the methods of
//! the [`LogControl1`] trait to call the corresponding log control methods in
//! the DBus callbacks.
//!
//! In addition to this core trait and related types, this crate also provides
//! some concrete helper functions to implement aspects of the log control
//! interface.
//!
//! [`DBUS_OBJ_PATH`] provides a constant for the DBUs object path the interface
//! must be served at according to the interface specification, in order to be
//! found by `systemdctl`.
//!
//! [`stderr_connected_to_journal`] determines whether the current process has
//! its stderr directly connected to the systemd journal (as for all processes
//! directly started via systemd units); in this case a log control implementation
//! should default to logging to the [`KnownLogTarget::Journal`] log target.
//! This function also helps to implement the [`KnownLogTarget::Auto`] target.
//!
//! ## Logging framework implementations and DBus frontends
//!
//! The following crates provides implementations of the [`LogControl1`] trait
//! for a certain logging framework:
//!
//! - [`logcontrol-tracing`](https://docs.rs/logcontrol-tracing) implements
//!   the log control interface on top of the [`tracing`](https://doc.rs/tracing)
//!   crate.
//!
//! These crates implement DBus frontends to actually expose an implementation
//! of the [`LogControl1`] trait over DBus:
//!
//! - [`logcontrol-zbus`](https://docs.rs/logcontrol-zbus) glues a [`LogControl1`]
//!   into the pure Rust DBus implementation [`zbus`](https://docs.rs/zbus).
//!
//! [logcontrol]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html

#![deny(warnings, clippy::all, missing_docs, missing_debug_implementations)]
#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter};
use std::os::{fd::AsFd, linux::fs::MetadataExt};

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

/// Known log targets documented in the log control interface or `systemctl(1)`.
///
/// Note that `systemctl` does not validate the log target; `systemctl service-log-target`
/// passes any given string to the service.
///
/// This enum represents all log targets documented at various places, and their
/// semantics.
///
/// Implementations of [`LogControl1`] can use this enum to parse known targets,
/// or entirely ignore it and handle the target themselves; the latter allows
/// services to implement their own proprietary log targets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnownLogTarget {
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
    /// Disable all logging.
    ///
    /// Note that even though this target is not documented in the logcontrol
    /// interface definition, the `systemctl(1)` manpage mentions it for the
    /// `service-log-target` command.
    Null,
    /// Automatically log to console or journal.
    ///
    /// If the stdout or stderr streams of the current process are
    /// connected to the systemd journal this is equivalent to `Journal`.
    /// Otherwise it's `Console`.
    ///
    /// See [`stderr_connected_to_journal`] for a function which implements this
    /// check.
    ///
    /// Note that even though this target is not documented in the logcontrol
    /// interface definition, the `systemctl(1)` manpage mentions it for the
    /// `service-log-target` command.
    Auto,
}

impl KnownLogTarget {
    /// Convert to the corresponding string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            KnownLogTarget::Console => "console",
            KnownLogTarget::Kmsg => "kmsg",
            KnownLogTarget::Journal => "journal",
            KnownLogTarget::Syslog => "syslog",
            KnownLogTarget::Null => "null",
            KnownLogTarget::Auto => "auto",
        }
    }
}

/// The log target was invalid.
#[derive(Debug, Clone, Error)]
#[error("Invalid log target: '{0}'")]
pub struct LogTargetParseError(String);

impl From<LogTargetParseError> for LogControl1Error {
    fn from(value: LogTargetParseError) -> Self {
        Self::UnsupportedLogTarget(value.0)
    }
}

impl TryFrom<&str> for KnownLogTarget {
    type Error = LogTargetParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "console" => Ok(KnownLogTarget::Console),
            "kmsg" => Ok(KnownLogTarget::Kmsg),
            "journal" => Ok(KnownLogTarget::Journal),
            "syslog" => Ok(KnownLogTarget::Syslog),
            "null" => Ok(KnownLogTarget::Null),
            "auto" => Ok(KnownLogTarget::Auto),
            _ => Err(LogTargetParseError(value.to_string())),
        }
    }
}

impl Display for KnownLogTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
    UnsupportedLogTarget(String),
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
    fn target(&self) -> &str;

    /// Set the target of the underlying log framework.
    ///
    /// Systemd documents some known targets both in the `LogControl1` interface
    /// definition, as well as in the `systemctl(1)` manpage.  The [`KnownLogTarget`]
    /// enum represents all these known targets.
    ///
    /// However, implementations are free to use their own proprietary targets;
    /// `systemctl service-log-target` actually forwards any given string to the
    /// service.
    ///
    /// It's a good idea though to support at least [`KnownLogTarget::Console`]
    /// and [`KnownLogTarget::Journal`].
    fn set_target<S: AsRef<str>>(&mut self, target: S) -> Result<(), LogControl1Error>;

    /// Get the syslog identifier.
    fn syslog_identifier(&self) -> &str;
}

/// The DBus object path a log control interface needs to be served on for systemd to find it.
///
/// The path is `/org/freedesktop/LogControl1`, as required by the interface specification.
pub static DBUS_OBJ_PATH: &str = "/org/freedesktop/LogControl1";

/// Whether the current process is directly connected to the systemd journal.
///
/// You can use this function to implement [`KnownLogTarget::Auto`].
///
/// Return `true` if the device and inode numbers of the [`std::io::stderr`]
/// file descriptor match the value of `$JOURNAL_STREAM` (see `systemd.exec(5)`).
/// Otherwise, return `false`.
pub fn stderr_connected_to_journal() -> bool {
    std::io::stderr()
        .as_fd()
        .try_clone_to_owned()
        .and_then(|fd| std::fs::File::from(fd).metadata())
        .map(|metadata| format!("{}:{}", metadata.st_dev(), metadata.st_ino()))
        .ok()
        .and_then(|stderr| {
            std::env::var_os("JOURNAL_STREAM").map(|s| s.to_string_lossy() == stderr.as_str())
        })
        .unwrap_or(false)
}

/// Determine the syslog identifier for this process.
///
/// This function obtains the syslog identifier from the file name of the
/// current executable, per [`std::env::current_exe`].
///
/// As such, it's a comparatively expensive function to call; implementations of
/// [`LogControl1`] should avoid calling it for every invocation, but instead
/// determine the identifier once upon construction and store it.
///
/// If it fails to determine the syslog identifier, i.e. when `current_exe`
/// returns an error, this function falls back to the empty string.
pub fn syslog_identifier() -> String {
    std::env::current_exe()
        .ok()
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        // If we fail to get the name of the current executable fall back to an empty string.
        .unwrap_or_else(String::new)
}
