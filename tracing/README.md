# logcontrol-tracing

[![Crates.io](https://img.shields.io/crates/v/logcontrol-tracing)](https://crates.io/crates/logcontrol-tracing)
[![docs.rs](https://img.shields.io/docsrs/logcontrol-tracing)](https://docs.rs/logcontrol-tracing)

[`tracing`][tracing] implementation for the [logcontrol] interface.

[tracing]: https://github.com/tokio-rs/tracing
[logcontrol]: https://codeberg.org/swsnr/logcontrol.rs

## Usage

```console
$ cargo add logcontrol-tracing
```

```rust
use std::error::Error;

use logcontrol_tracing::{PrettyLogControl1LayerFactory, TracingLogControl1};
use logcontrol_zbus::ConnectionBuilderExt;
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
        .serve_log_control(logcontrol_zbus::LogControl1::new(control))?
        .build()
        .await?;

    loop {
        // Service event loop
    }
}
```
