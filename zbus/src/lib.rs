//! A [`logcontrol::LogControl1`] frontend with [`zbus`].
//!
//! [`LogControl1`] provides the D-Bus interface implementation.  It receives the
//!  underlying [`logcontrol::LogControl1`] as sole argument and exposes it
//! over D-Bus, as a standard zbus D-Bus interface:
//!
//! ```ignore
//! use async_io::block_on;
//! use logcontrol_zbus::{LogControl1, ConnectionBuilderExt};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let control = create_log_control();
//!
//!     block_on(async move {
//!          let _conn = zbus::ConnectionBuilder::session()?
//!              .name("de.swsnr.logcontrol.SimpleServerExample")?
//!              .serve_log_control(LogControl1::new(control))?
//!              .build()
//!              .await?;
//!
//!          // Do other things or go to wait forever
//!          std::future::pending::<()>().await;
//!
//!          Ok(())
//!     })
//! }
//! ```
//!
//! Note that for `systemctl` to find the log control interface with
//! `systemctl service-log-level` and `systemctl service-log-target` you need
//! to make sure that
//!
//! - the interface is registered under [`logcontrol::DBUS_OBJ_PATH`], and
//! - the unit file provides the claimed bus name in the `BusName` attribute.
//!
//! Otherwise systemd will not be able to change the log level or target.

#![deny(warnings, clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

use logcontrol::{LogControl1Error, LogLevel};
use zbus::interface;

pub use logcontrol;
pub use logcontrol::DBUS_OBJ_PATH;

fn to_fdo_error(error: LogControl1Error) -> zbus::fdo::Error {
    #[allow(clippy::enum_glob_use)]
    use LogControl1Error::*;
    match error {
        UnsupportedLogLevel(_) | UnsupportedLogTarget(_) => {
            zbus::fdo::Error::NotSupported(error.to_string())
        }
        InputOutputError(error) => zbus::fdo::Error::IOError(error.to_string()),
        Failure(msg) => zbus::fdo::Error::Failed(msg),
    }
}

/// A [`zbus`] frontend for [`logcontrol::LogControl1`].
///
/// See [crate documentation][`crate`] for an example and further
/// usage instructions.
pub struct LogControl1<C>
where
    C: logcontrol::LogControl1 + Send + Sync,
{
    control: C,
}

impl<C> LogControl1<C>
where
    C: logcontrol::LogControl1 + Send + Sync + 'static,
{
    /// Create a new D-Bus interface around the given log control interface.
    pub fn new(control: C) -> Self {
        Self { control }
    }
}

/// The log control interface.
///
/// See <https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html>.
#[interface(name = "org.freedesktop.LogControl1")]
impl<C> LogControl1<C>
where
    C: logcontrol::LogControl1 + Send + Sync + 'static,
{
    /// Get the currently configured log level.
    #[zbus(property)]
    fn log_level(&self) -> String {
        self.control.level().to_string()
    }

    /// Set the new log level.
    #[zbus(property)]
    fn set_log_level(&mut self, level: &str) -> zbus::fdo::Result<()> {
        let level = LogLevel::try_from(level)
            .map_err(|error| zbus::fdo::Error::InvalidArgs(error.to_string()))?;
        self.control.set_level(level).map_err(to_fdo_error)
    }

    /// Get the currently configured log target.
    #[zbus(property)]
    fn log_target(&self) -> String {
        self.control.target().to_string()
    }

    /// Change the log target.
    #[zbus(property)]
    fn set_log_target(&mut self, target: String) -> zbus::fdo::Result<()> {
        self.control.set_target(target).map_err(to_fdo_error)
    }

    /// Get the syslog identifier used by the service.
    #[zbus(property)]
    fn syslog_identifier(&self) -> &str {
        self.control.syslog_identifier()
    }
}

/// Extend `ConnectionBuilder` to serve log control interfaces.
pub trait ConnectionBuilderExt {
    /// Serve the given log control interface on this connection builder.
    ///
    /// # Errors
    ///
    /// Return an error if registering the log control object failed.
    fn serve_log_control<C>(self, iface: LogControl1<C>) -> zbus::Result<Self>
    where
        Self: Sized,
        C: logcontrol::LogControl1 + Send + Sync + 'static;
}

impl ConnectionBuilderExt for zbus::connection::Builder<'_> {
    fn serve_log_control<C>(self, iface: LogControl1<C>) -> zbus::Result<Self>
    where
        C: logcontrol::LogControl1 + Send + Sync + 'static,
    {
        self.serve_at(DBUS_OBJ_PATH, iface)
    }
}

impl ConnectionBuilderExt for zbus::blocking::connection::Builder<'_> {
    fn serve_log_control<C>(self, iface: LogControl1<C>) -> zbus::Result<Self>
    where
        C: logcontrol::LogControl1 + Send + Sync + 'static,
    {
        self.serve_at(DBUS_OBJ_PATH, iface)
    }
}
