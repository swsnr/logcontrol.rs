//! A simple zbus server which exposes the log control interface.
//!
//! Run as an ad-hoc service via
//!
//! ```
//! $ systemd-run --user --pty \
//!     --service-type=dbus --unit=log-control-example.service \
//!     --property=BusName=de.swsnr.logcontrol.LogServerExample \
//!     --setenv=RUST_LOG=trace \
//!     ./target/debug/examples/zbus_log
//! ```
//!
//! Then use `systemctl --user service-log-level log-control-example.service`
//! or `systemctl --user service-log-target log-control-example.service` to test
//! the interface.
//!
//! To see its log messages in the system journal, use `journalctl --user
//! -u log-control-example.service`.

use std::error::Error;
use std::time::Duration;

use log::{info, warn};
use logcontrol_log::{LogController, LogFactory};
use logcontrol_zbus::ConnectionBuilderExt;

struct Factory;

impl LogFactory for Factory {
    fn create_console_log(&self) -> Result<Box<dyn log::Log>, logcontrol::LogControl1Error> {
        Ok(Box::new(env_logger::Builder::from_default_env().build()))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let control = LogController::install_auto(Factory, log::Level::Info)?;
    log::set_max_level(log::LevelFilter::Trace);

    async_io::block_on(async move {
        let _conn = zbus::connection::Builder::session()?
            .name("de.swsnr.logcontrol.LogServerExample")?
            .serve_log_control(logcontrol_zbus::LogControl1::new(control))?
            .build()
            .await?;

        loop {
            async_io::Timer::after(Duration::from_secs(5)).await;
            info!("An message at info level");
            async_io::Timer::after(Duration::from_secs(1)).await;
            warn!("An message at warning level");
        }
    })
}
