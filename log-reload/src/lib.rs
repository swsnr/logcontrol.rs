//! A [`log::Log`] implementation which dynamically reloads inner loggers.
//!
//! [`ReloadLog`] wraps an inner logger and provides a [`ReloadHandle`] to
//! dynamically replace or modify the inner logger.
//!
//! This allows programs to dynamically change the log level or log target at
//! runtime.

#![deny(warnings, clippy::all, clippy::pedantic, missing_docs)]
#![forbid(unsafe_code)]

use std::sync::{Arc, RwLock, Weak};

use log::Log;

/// Filter an underlying logger by a given max level.
///
/// Only forward log events whose log level is smaller or equal than the
/// configured level to the underlying logger.
#[derive(Debug)]
pub struct LevelFilter<T> {
    level: log::Level,
    logger: T,
}

impl<T> LevelFilter<T> {
    /// Create a new level filter with the given max `level` around the given `logger`.
    pub fn new(level: log::Level, logger: T) -> Self {
        Self { level, logger }
    }

    /// Get the current log level.
    pub fn level(&self) -> log::Level {
        self.level
    }

    /// Change the maximum log level.
    pub fn set_level(&mut self, level: log::Level) {
        self.level = level;
    }

    fn level_passes(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    /// Get a reference to the inner unfiltered logger.
    pub fn inner(&self) -> &T {
        &self.logger
    }

    /// Replace the inner logger.
    pub fn set_inner(&mut self, logger: T) {
        self.logger = logger;
    }
}

impl<T: Log> log::Log for LevelFilter<T> {
    /// Wether this logger is enabled.
    ///
    /// Return `true` if the log level in `metadata` is less then the level of
    /// the given `metadata`, and the underlying logger is enabled.
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.level_passes(metadata) && self.logger.enabled(metadata)
    }

    /// Forward a log `record` to the underlying logger if it passes the level filter.
    fn log(&self, record: &log::Record) {
        if self.level_passes(record.metadata()) {
            self.logger.log(record);
        }
    }

    /// Flush the underlying logger.
    fn flush(&self) {
        self.logger.flush();
    }
}

/// A logger which can dynamically reload an inner logger.
///
/// This enables applications to dyanmically change e.g. the log output or
/// log level.
#[derive(Debug)]
pub struct ReloadLog<T> {
    underlying: Arc<RwLock<T>>,
}

impl<T> ReloadLog<T> {
    /// Create a new reloadable logger over the given `logger`.
    pub fn new(logger: T) -> Self {
        Self {
            underlying: Arc::new(RwLock::new(logger)),
        }
    }

    /// Obtain a handle to reload or modify the inner logger.
    #[must_use]
    pub fn handle(&self) -> ReloadHandle<T> {
        ReloadHandle {
            underlying: Arc::downgrade(&self.underlying),
        }
    }
}

impl<T: Log> Log for ReloadLog<T> {
    /// Whether the underlying logger is enabled.
    ///
    /// Always return `false` if the [`RwLock`] protecting the inner logger is poisoned,
    /// because we can't trust that the inner logger is valid if a panic occurred
    /// while it was modified, so we indicate that this logger shouldn't be used at all.
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.underlying.read().is_ok_and(|l| l.enabled(metadata))
    }

    /// Log the given `record` with the inner logger.
    ///
    /// If the [`RwLock`] protecting the inner logger is poisoned do nothing,
    /// because we can't trust that the inner logger is valid if a panic occurred
    /// while it was modified.  The `record` is likely lost in this case.
    fn log(&self, record: &log::Record) {
        // We can't reasonably do anything if the lock is poisoned so we ignore the result
        let _ = self.underlying.read().map(|l| l.log(record));
    }

    /// Flush the inner logger
    ///
    /// If the [`RwLock`] protecting the inner logger is poisoned do nothing,
    /// because we can't trust that the inner logger is valid if a panic occurred
    /// while it was modified.
    fn flush(&self) {
        // We can't reasonably do anything if the lock is poisoned so we ignore the result
        let _ = self.underlying.read().map(|l| l.flush());
    }
}

/// An error which occurred while reloading the logger.
#[derive(Debug, Clone, Copy)]
pub enum ReloadError {
    /// The logger referenced by the reload handle was dropped meanwhile.
    Gone,
    /// The lock protecting the inner logger referenced by the reload is poisoned.
    ///
    /// Note that this is an error because we currently can't recover from a
    /// poisoned [`RwLock`] in stable rust, as [`RwLock::clear_poison`] is still
    /// experimental.
    ///
    /// Once this method stabilizes reloading can simply overwrite any poisoned
    /// data and clear the poison flag to resume logging.  At this point this
    /// error condition will be removed.
    ///
    /// See <https://github.com/rust-lang/rust/issues/96469> for stabilization of
    /// [`RwLock::clear_poison`].
    Poisoned,
}

impl std::fmt::Display for ReloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReloadError::Gone => write!(f, "Referenced logger was dropped"),
            ReloadError::Poisoned => write!(f, "Lock poisoned"),
        }
    }
}

impl std::error::Error for ReloadError {}

/// A handle to reload a logger inside a [`ReloadLog`].
#[derive(Debug, Clone)]
pub struct ReloadHandle<T> {
    underlying: Weak<RwLock<T>>,
}

impl<T> ReloadHandle<T> {
    /// Replace the inner logger.
    ///
    /// This replaces the inner logger of the referenced [`ReloadLog`] with the given `logger`.
    ///
    /// # Errors
    ///
    /// Return [`ReloadError::Gone`] if the target logger was dropped, and
    /// [`ReloadError::Poisoned`] if the reload lock is poisoned.
    pub fn replace(&self, logger: T) -> Result<(), ReloadError> {
        let lock = self.underlying.upgrade().ok_or(ReloadError::Gone)?;
        // TODO: Overwrite and clear poison, once clear_poison() is stabilized
        // See https://github.com/rust-lang/rust/issues/96469
        let mut guard = lock.write().map_err(|_| ReloadError::Poisoned)?;
        *guard = logger;
        Ok(())
    }

    /// Modify the inner logger.
    ///
    /// Call the given function with a mutable reference to the logger.  Note that
    /// a lock is held while invoking `f`, so no log messages will be processed
    /// until `f` returns.
    ///
    /// If `f` panics this lock gets poisoned which effectively disables the logger.
    ///
    /// # Errors
    ///
    /// Return [`ReloadError::Poisoned`] if the reload lock is poisoned.
    pub fn modify<F>(&self, f: F) -> Result<(), ReloadError>
    where
        F: FnOnce(&mut T),
    {
        let lock = self.underlying.upgrade().ok_or(ReloadError::Gone)?;
        // TODO: Overwrite and clear poison, once clear_poison() is stabilized
        // See https://github.com/rust-lang/rust/issues/96469
        let mut guard = lock.write().map_err(|_| ReloadError::Poisoned)?;
        f(&mut *guard);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{LevelFilter, ReloadLog};
    use log::{Log, Record};
    use similar_asserts::assert_eq;
    use std::sync::{Arc, Mutex};

    struct CollectMessages {
        messages: Mutex<Vec<String>>,
    }

    impl CollectMessages {
        fn new() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
            }
        }
    }

    impl Log for CollectMessages {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            let mut guard = self.messages.try_lock().unwrap();
            guard.push(format!("{}", record.args()));
        }

        fn flush(&self) {}
    }

    #[test]
    fn sanity_check_log_level_ordering() {
        use log::Level;

        assert!(Level::Error <= Level::Warn);
        assert!(Level::Warn <= Level::Warn);
        assert!(Level::Debug >= Level::Warn);
    }

    #[test]
    fn level_filter() {
        let collect_logs = Arc::new(CollectMessages::new());
        let mut filter = LevelFilter::new(log::Level::Warn, collect_logs.clone());

        for level in log::Level::iter() {
            filter.log(
                &Record::builder()
                    .level(level)
                    .args(format_args!("{level}"))
                    .build(),
            );
        }
        let mut messages = collect_logs.messages.try_lock().unwrap();
        assert_eq!(*messages, vec!["ERROR", "WARN"]);
        messages.clear();
        drop(messages);

        filter.set_level(log::Level::Debug);

        for level in log::Level::iter() {
            filter.log(
                &Record::builder()
                    .level(level)
                    .args(format_args!("{level}"))
                    .build(),
            );
        }
        let messages = collect_logs.messages.try_lock().unwrap();
        assert_eq!(*messages, &["ERROR", "WARN", "INFO", "DEBUG"]);
    }

    #[test]
    fn reloadlog_replace() {
        let collect_logs_1 = Arc::new(CollectMessages::new());
        let collect_logs_2 = Arc::new(CollectMessages::new());

        let reload_log = ReloadLog::new(collect_logs_1.clone());
        let reload_handle = reload_log.handle();

        reload_log.log(&Record::builder().args(format_args!("Message 1")).build());

        reload_handle.replace(collect_logs_2.clone()).unwrap();

        reload_log.log(&Record::builder().args(format_args!("Message 2")).build());

        let messages_1 = collect_logs_1.messages.try_lock().unwrap();
        let messages_2 = collect_logs_2.messages.try_lock().unwrap();
        assert_eq!(*messages_1, &["Message 1"]);
        assert_eq!(*messages_2, &["Message 2"]);
    }

    #[test]
    fn reloadlog_modify() {
        let collect_logs = Arc::new(CollectMessages::new());

        let reload_log = ReloadLog::new(collect_logs.clone());
        let reload_handle = reload_log.handle();

        reload_log.log(&Record::builder().args(format_args!("Message 1")).build());
        let messages = collect_logs.messages.try_lock().unwrap();
        assert_eq!(*messages, &["Message 1"]);
        drop(messages);

        // Clear the message store through the reload handle.
        reload_handle
            .modify(|l| l.messages.try_lock().unwrap().clear())
            .unwrap();

        // At this point the first message doesn't appear anymore.
        reload_log.log(&Record::builder().args(format_args!("Message 2")).build());
        let messages = collect_logs.messages.try_lock().unwrap();
        assert_eq!(*messages, &["Message 2"]);
    }
}
