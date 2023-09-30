//! A [`logcontrol::LogControl1`] frontend with [`zbus`].
//!
//! [`LogControl1`] provides the DBus interface implementation.  It receives the
//!  underlying [`logcontrol::LogControl1`] as sole argument and exposes it
//! over DBus, as a standard zbus DBus interface:
//!
//! ```ignore
//! #[async_std::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let control = create_log_control();
//!     let _conn = zbus::ConnectionBuilder::session()?
//!         .name("de.swsnr.logcontrol.SimpleServerExample")?
//!         .serve_at(
//!             logcontrol::DBUS_OBJ_PATH,
//!             logcontrol_zbus::LogControl1::new(control),
//!         )?
//!         .build()
//!         .await?;
//!
//!     // Do other things or go to wait forever
//!     std::future::pending::<()>().await;
//!
//!     Ok(())
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

#![deny(warnings, clippy::all)]
#![forbid(unsafe_code)]

use logcontrol::{LogControl1Error, LogLevel};
use zbus::dbus_interface;

pub use logcontrol;
pub use logcontrol::DBUS_OBJ_PATH;

fn to_fdo_error(error: LogControl1Error) -> zbus::fdo::Error {
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
/// See [crate documentation][`logcontrol-zbus`] for an example and further
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
    /// Create a new DBus interface around the given log control interface.
    pub fn new(control: C) -> Self {
        Self { control }
    }
}

/// The log control interface.
///
/// See <https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html>.
#[dbus_interface(name = "org.freedesktop.LogControl1")]
impl<C> LogControl1<C>
where
    C: logcontrol::LogControl1 + Send + Sync + 'static,
{
    /// Get the currently configured log level.
    #[dbus_interface(property)]
    fn log_level(&self) -> String {
        self.control.level().to_string()
    }

    /// Set the new log level.
    #[dbus_interface(property)]
    fn set_log_level(&mut self, level: String) -> zbus::fdo::Result<()> {
        let level = LogLevel::try_from(level.as_str())
            .map_err(|error| zbus::fdo::Error::InvalidArgs(error.to_string()))?;
        self.control.set_level(level).map_err(to_fdo_error)
    }

    /// Get the currently configured log target.
    #[dbus_interface(property)]
    fn log_target(&self) -> String {
        self.control.target().to_string()
    }

    /// Change the log target.
    #[dbus_interface(property)]
    async fn set_log_target(&mut self, target: String) -> zbus::fdo::Result<()> {
        self.control.set_target(target).map_err(to_fdo_error)
    }

    /// Get the syslog identifier used by the service.
    #[dbus_interface(property)]
    fn syslog_identifier(&self) -> &str {
        self.control.syslog_identifier()
    }
}
