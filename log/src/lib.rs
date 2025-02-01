//! A [`LogControl1`] implementation for [`log`].
//!
//! [`LogController`] provides a [`LogControl1`] implementation on top of [`log`]
//! which uses the [`log_reload`] crate to dynamically switch loggers and levels
//! depending on the target and level selected over the log control interface.
//!
//! It uses a [`LogFactory`] implementation to create the actual [`Log`] instances
//! each time the log target is changed.  This crates provides _no_ default
//! implementation of this trait; users have to provide an implementation on
//! their own.  This avoids a dependency on any specific log implementation for
//! the `console` target.
//!
//! For the `journal` target this crate uses the [`systemd_journal_logger`] crate.
//!
//! See [`LogController::install_auto`] for the recommended entry point to this crate.

#![deny(warnings, clippy::all, clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

use log::Log;
use log_reload::LevelFilter;
use log_reload::ReloadHandle;
use log_reload::ReloadLog;
use logcontrol::KnownLogTarget;
use logcontrol::LogControl1;
use logcontrol::LogControl1Error;
use logcontrol::LogLevel;

pub use logcontrol;
pub use logcontrol::stderr_connected_to_journal;
pub use logcontrol::syslog_identifier;
use systemd_journal_logger::JournalLog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupportedLogTarget {
    Console,
    Journal,
}

impl From<SupportedLogTarget> for KnownLogTarget {
    fn from(value: SupportedLogTarget) -> Self {
        match value {
            SupportedLogTarget::Console => KnownLogTarget::Console,
            SupportedLogTarget::Journal => KnownLogTarget::Journal,
        }
    }
}

fn from_known_log_target(
    target: KnownLogTarget,
    connected_to_journal: bool,
) -> Result<SupportedLogTarget, LogControl1Error> {
    match target {
        KnownLogTarget::Auto if connected_to_journal => Ok(SupportedLogTarget::Journal),
        KnownLogTarget::Auto | KnownLogTarget::Console => Ok(SupportedLogTarget::Console),
        KnownLogTarget::Journal => Ok(SupportedLogTarget::Journal),
        other => Err(LogControl1Error::UnsupportedLogTarget(
            other.as_str().to_string(),
        )),
    }
}

/// Convert [`logcontrol::LogLevel`] to [`log::Level`].
///
/// Return an error if the systemd log level is not supported, i.e. does not map to a
/// corresponding [`log::Level`].
///
/// # Errors
///
/// Return [`LogControl1Error::UnsupportedLogLevel`] if the log `level` from the
/// logcontrol interface does not map to a [`log::Level`].
pub fn from_log_level(level: LogLevel) -> Result<log::Level, LogControl1Error> {
    match level {
        LogLevel::Err => Ok(log::Level::Error),
        LogLevel::Warning => Ok(log::Level::Warn),
        LogLevel::Notice => Ok(log::Level::Info),
        LogLevel::Info => Ok(log::Level::Debug),
        LogLevel::Debug => Ok(log::Level::Trace),
        unsupported => Err(LogControl1Error::UnsupportedLogLevel(unsupported)),
    }
}

/// Convert [`log::Level`] to [`logcontrol::LogLevel`].
fn to_log_level(level: log::Level) -> LogLevel {
    match level {
        log::Level::Error => LogLevel::Err,
        log::Level::Warn => LogLevel::Warning,
        log::Level::Info => LogLevel::Notice,
        log::Level::Debug => LogLevel::Info,
        log::Level::Trace => LogLevel::Debug,
    }
}

fn create_logger<F: LogFactory>(
    target: SupportedLogTarget,
    factory: &F,
    syslog_identifier: &str,
) -> Result<Box<dyn Log>, LogControl1Error> {
    match target {
        SupportedLogTarget::Console => factory.create_console_log(),
        SupportedLogTarget::Journal => factory.create_journal_log(syslog_identifier.to_string()),
    }
}

/// A factory for log implementations.
pub trait LogFactory {
    /// Create a logger for the console log target.
    ///
    /// # Errors
    ///
    /// Return an error if creating the logger failed.
    fn create_console_log(&self) -> Result<Box<dyn Log>, LogControl1Error>;

    /// Create a logger for journal log target.
    ///
    /// The implementation should use `syslog_identifier` for the corresponding journal field.
    ///
    /// The default implementation creates a [`systemd_journal_logger::JournalLog`].
    ///
    /// # Errors
    ///
    /// Return [`LogControl1Error::InputOutputError`] if journald is unavailable.
    fn create_journal_log(
        &self,
        syslog_identifier: String,
    ) -> Result<Box<dyn Log>, LogControl1Error> {
        Ok(Box::new(
            JournalLog::empty()?.with_syslog_identifier(syslog_identifier),
        ))
    }
}

/// The type of a controlled [`log::Log`].
pub type ControlledLog = ReloadLog<LevelFilter<Box<dyn Log>>>;

/// A [`LogControl1`] implementation for [`log`].
///
/// This implementation creates a [`log::Log`] implementation whose level and
/// underlying logger can be dynamically reconfigured through the [`LogControl1`]
/// interface.  It uses a [`ReloadLog`] together with a [`LevelFilter`] under
/// the hood.
///
/// Currently, this implementation only supports for following [`KnownLogTarget`]s:
///
/// - [`KnownLogTarget::Console`]
/// - [`KnownLogTarget::Journal`]
/// - [`KnownLogTarget::Auto`]
///
/// Any other target fails with [`LogControl1Error::UnsupportedLogTarget`].
pub struct LogController<F: LogFactory> {
    /// The reload handler.
    handle: ReloadHandle<LevelFilter<Box<dyn Log>>>,
    /// The factory to create loggers with when switching targets.
    factory: F,
    /// Whether the current process is connnected to the systemd journal.
    connected_to_journal: bool,
    /// The syslog identifier used for logging.
    syslog_identifier: String,
    /// The current level active in the level layer.
    level: LogLevel,
    /// The current target active in the target layer.
    target: SupportedLogTarget,
}

impl<F: LogFactory> LogController<F> {
    /// Create a new logger which can be controlled through the log control interface.
    ///
    /// `factory` creates the inner [`log::Log`] instances for the selected `target` which
    /// denotes the initial log target.  The `factory` is invoked whenever the log target
    /// is changed, to create a new logger for the corresponding target.  See
    /// [`LogController`] for supported log targets.
    ///
    /// `connected_to_journal` indicates whether this process is connected to the systemd
    /// journal. Set to `true` to make [`KnownLogTarget::Auto`] use [`KnownLogTarget::Journal`],
    /// otherwise it uses [`KnownLogTarget::Console`].
    ///
    /// `level` denotes the default tracing log level to start with.
    ///
    /// `syslog_identifier` is passed to [`LogFactory::create_journal_log`]
    /// for use as `SYSLOG_IDENTIFIER` journal field.
    ///
    /// Returns an error if `target` is not supported, of if creating a layer fails,
    ///
    /// # Errors
    ///
    /// Return a [`LogControl1Error::UnsupportedLogTarget`] if `target` is
    /// not supported, and [`LogControl1Error::InputOutputError`] if creating
    /// the logger for `target` failed, e.g. when selecting [`KnownLogTarget::Journal`]
    /// on a system where journald is not running, or inside a container which
    /// has no direct access to the journald socket.
    pub fn new(
        factory: F,
        connected_to_journal: bool,
        syslog_identifier: String,
        target: KnownLogTarget,
        level: log::Level,
    ) -> Result<(Self, ControlledLog), LogControl1Error> {
        let log_target = from_known_log_target(target, connected_to_journal)?;
        let inner_logger = create_logger(log_target, &factory, &syslog_identifier)?;
        let log = ReloadLog::new(LevelFilter::new(level, inner_logger));
        let control = Self {
            handle: log.handle(),
            factory,
            connected_to_journal,
            syslog_identifier,
            level: to_log_level(level),
            target: log_target,
        };
        Ok((control, log))
    }

    /// Create a new logger which can be controlled through the log control interface, using automatic defaults.
    ///
    /// Use [`logcontrol::syslog_identifier()`] as the syslog identifier, and
    /// determine the initial log target automatically according to
    /// [`logcontrol::stderr_connected_to_journal()`].
    ///
    /// `level` denotes the initial level; for `factory` and returned errors,
    ///  see [`Self::new`].
    ///
    /// # Errors
    ///
    /// Return [`LogControl1Error::InputOutputError`] if journald is not
    /// available, but should have been.  This will only happen on a broken
    /// system.
    pub fn new_auto(
        factory: F,
        level: log::Level,
    ) -> Result<(Self, ControlledLog), LogControl1Error> {
        Self::new(
            factory,
            logcontrol::stderr_connected_to_journal(),
            logcontrol::syslog_identifier(),
            KnownLogTarget::Auto,
            level,
        )
    }

    /**
     * Create and install a controlled logger, with automatic defaults.
     *
     * See [`Self::new_auto`] for arguments.
     *
     * # Errors
     *
     * See [`Self::new_auto`] for errors. Additionally, this function fails with
     * [`LogControl1Error::Failure`] if [`log::set_boxed_logger`] fails.
     */
    pub fn install_auto(factory: F, level: log::Level) -> Result<Self, LogControl1Error> {
        let (control, logger) = Self::new_auto(factory, level)?;
        log::set_boxed_logger(Box::new(logger))
            .map_err(|error| LogControl1Error::Failure(format!("{error}")))?;
        Ok(control)
    }
}

impl<F: LogFactory> LogControl1 for LogController<F> {
    fn level(&self) -> logcontrol::LogLevel {
        self.level
    }

    fn set_level(
        &mut self,
        level: logcontrol::LogLevel,
    ) -> Result<(), logcontrol::LogControl1Error> {
        let log_level = from_log_level(level)?;
        self.handle
            .modify(|l| l.set_level(log_level))
            .map_err(|error| {
                LogControl1Error::Failure(format!("Failed to change level to {level}: {error}"))
            })?;
        self.level = level;
        Ok(())
    }

    fn target(&self) -> &str {
        KnownLogTarget::from(self.target).as_str()
    }

    fn set_target<S: AsRef<str>>(&mut self, target: S) -> Result<(), logcontrol::LogControl1Error> {
        let log_target = from_known_log_target(
            KnownLogTarget::try_from(target.as_ref())?,
            self.connected_to_journal,
        )?;
        let new_logger = create_logger(log_target, &self.factory, &self.syslog_identifier)?;
        self.handle
            .modify(|l| l.set_inner(new_logger))
            .map_err(|error| {
                LogControl1Error::Failure(format!(
                    "Failed to change log target to {}: {error}",
                    target.as_ref()
                ))
            })?;
        self.target = log_target;
        Ok(())
    }

    fn syslog_identifier(&self) -> &str {
        &self.syslog_identifier
    }
}
