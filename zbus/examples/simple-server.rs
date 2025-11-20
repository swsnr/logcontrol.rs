//! A simple zbus server which exposes the log control interface.
//!
//! Run as an ad-hoc service via
//!
//! ```
//! $ systemd-run --user --pty \
//!     --service-type=dbus --unit=log-control-example.service \
//!     --property=BusName=de.swsnr.logcontrol.SimpleServerExample \
//!     ./target/debug/examples/simple-server
//! ```
//!
//! Then use `systemctl --user service-log-level log-control-example.service`
//! or `systemctl --user service-log-target log-control-example.service` to test
//! the interface.

use std::{error::Error, future::pending};

use logcontrol::LogControl1;

struct DummyLogControl {
    level: logcontrol::LogLevel,
    target: logcontrol::KnownLogTarget,
}

impl LogControl1 for DummyLogControl {
    fn level(&self) -> logcontrol::LogLevel {
        self.level
    }

    fn set_level(
        &mut self,
        level: logcontrol::LogLevel,
    ) -> Result<(), logcontrol::LogControl1Error> {
        eprintln!("Setting level to {level}");
        self.level = level;
        Ok(())
    }

    fn target(&self) -> &str {
        self.target.as_str()
    }

    fn set_target<S: AsRef<str>>(&mut self, target: S) -> Result<(), logcontrol::LogControl1Error> {
        eprintln!("Setting target to {}", target.as_ref());
        self.target = target.as_ref().try_into()?;
        Ok(())
    }

    fn syslog_identifier(&self) -> &str {
        "foo"
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let control = DummyLogControl {
        level: logcontrol::LogLevel::Info,
        target: logcontrol::KnownLogTarget::Console,
    };

    async_io::block_on(async move {
        let _conn = zbus::connection::Builder::session()?
            .name("de.swsnr.logcontrol.SimpleServerExample")?
            .serve_at(
                logcontrol::DBUS_OBJ_PATH,
                logcontrol_zbus::LogControl1::new(control),
            )?
            .build()
            .await?;

        // Do other things or go to wait forever
        pending::<()>().await;

        Ok(())
    })
}
