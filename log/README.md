# logcontrol-log

[![Crates.io](https://img.shields.io/crates/v/logcontrol-log)](https://crates.io/crates/logcontrol-log)
[![docs.rs](https://img.shields.io/docsrs/logcontrol-log)](https://docs.rs/logcontrol-log)

[`log`][log] implementation for the [logcontrol] interface.

[log]: https://github.com/rust-lang/log
[logcontrol]: https://github.com/swsnr/logcontrol.rs

## Usage

```console
$ cargo add logcontrol-log
```

```rust
use std::error::Error;

use logcontrol_log::{LogController, LogFactory};
use logcontrol_zbus::ConnectionBuilderExt;
use zbus::ConnectionBuilder;

struct Factory;

impl LogFactory for Factory {
    fn create_console_log(&self) -> Result<Box<dyn log::Log>, logcontrol::LogControl1Error> {
        Ok(Box::new(env_logger::Builder::from_default_env().build()))
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let control = LogController::install_auto(Factory, log::Level::Info)?;
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
