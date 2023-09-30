//! A simple zbus server which exposes the log control interface.
//!
//! Run as an ad-hoc service via
//!
//! ```
//! $ systemd-run --user --pty \
//!     --service-type=dbus --unit=log-control-example.service \
//!     --property=BusName=de.swsnr.logcontrol.TracingServerExample \
//!     ./target/debug/examples/tracing-server
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

use logcontrol_tracing::{PrettyLogControl1LayerFactory, TracingLogControl1};
use tracing::{event, Level};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use zbus::ConnectionBuilder;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup env filter for convenient log control on console
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().ok();
    // If an env filter is set with $RUST_LOG use the lowest level as default for the control part,
    // to make sure the env filter takes precedence initially.
    let default_level = if env_filter.is_some() {
        Level::TRACE
    } else {
        Level::INFO
    };
    let (control, control_layer) =
        TracingLogControl1::new_auto(PrettyLogControl1LayerFactory, default_level)?;
    let subscriber = Registry::default().with(env_filter).with(control_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let _conn = ConnectionBuilder::session()?
        .name("de.swsnr.logcontrol.TracingServerExample")?
        .serve_at(
            logcontrol::DBUS_OBJ_PATH,
            logcontrol_zbus::LogControl1::new(control),
        )?
        .build()
        .await?;

    loop {
        async_std::task::sleep(Duration::from_secs(5)).await;
        event!(Level::INFO, "An message at info level");
        async_std::task::sleep(Duration::from_secs(1)).await;
        event!(Level::WARN, "An message at warning level");
    }
}
