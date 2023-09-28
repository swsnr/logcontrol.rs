//! Types for systemd's [logcontrol] interface.
//!
//! [logcontrol]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html#

#![deny(warnings, clippy::all, missing_docs, missing_debug_implementations)]
#![forbid(unsafe_code)]

use std::fmt::{Display, Formatter};

use thiserror::Error;

/// A syslog log level as used by the systemd log control interface.
///
/// See [POSIX syslog](https://pubs.opengroup.org/onlinepubs/9699919799.2018edition/functions/syslog.html)
/// or `syslog(3)` for more information.
#[derive(Debug, Copy, Clone)]
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

/// Errors w
#[derive(Debug, Copy, Clone, Error)]
pub enum LogLevelParseError {
    /// The log level was not valid.
    #[error("Invalid log level")]
    InvalidLogLevel,
}

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
            _ => Err(LogLevelParseError::InvalidLogLevel),
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
    /// The regular console, i.e. stdout or stderr.
    Console,
    /// The kernel ring message buffer.
    ///
    /// Normally not used by userspace services.
    Kmsg,
    /// The direct interface to the systemd journal.
    ///
    /// Prefer this over `Syslog`, and over console logging,
    /// if the process runs under systemd, because this interface
    /// retains all structured data.
    Journal,
    /// The legacy syslog interface.
    ///
    /// Services which use systemd should prefer the `Journal` interface.
    Syslog,
    /// Disable all logging.
    Null,
    /// Automatically log to console or journal.
    ///
    /// If the stdout or stderr streams of the current process are
    /// connected to the systemd journal this is equivalent to `Journal`.
    /// Otherwise it's `Console`.
    Auto,
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
            "null" => Ok(LogTarget::Null),
            "auto" => Ok(LogTarget::Auto),
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
            LogTarget::Null => "null",
            LogTarget::Auto => "auto",
        };
        write!(f, "{target}")
    }
}
