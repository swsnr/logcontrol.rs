# logcontrol.rs

[![Crates.io](https://img.shields.io/crates/v/logcontrol)](https://crates.io/crates/logcontrol)
[![docs.rs](https://img.shields.io/docsrs/logcontrol)](https://docs.rs/logcontrol)

Types and implementations for systemd's [logcontrol] interface.

This interface provides means to change logging behaviour of system services at runtime, over D-Bus, or via `systemctl service-log-level` or `systemctl service-log-target`.

This repository provides a collection of traits of basic types and implementations of this interface:

- `logcontrol` contains the basic types and defines an abstract trait for the interface.
- [`logcontrol-tracing`](https://github.com/swsnr/logcontrol.rs/tree/main/tracing) provides a logcontrol backend implementation for the [`tracing`][tracing] library.
- [`logcontrol-zbus`](https://github.com/swsnr/logcontrol.rs/tree/main/zbus) provides a DBus interface implementation for [`zbus`][zbus] DBus framework.

[logcontrol]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html#
[tracing]: https://github.com/tokio-rs/tracing
[zbus]: https://github.com/dbus2/zbus

## Usage

```console
$ cargo add logcontrol-tracing
$ cargo add logcontrol-zbus
```

```rust
use std::error::Error;
use std::time::Duration;

use logcontrol_tracing::{PrettyLogControl1LayerFactory, TracingLogControl1};
use logcontrol_zbus::{ConnectionBuilderExt, LogControl1};
use tracing::{event, Level};
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;
use zbus::ConnectionBuilder;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (control, control_layer) =
        TracingLogControl1::new_auto(PrettyLogControl1LayerFactory, Level::INFO)?;
    let subscriber = Registry::default().with(control_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let _conn = ConnectionBuilder::session()?
        .name("com.example.Foo")?
        .serve_log_control(LogControl1::new(control))?
        .build()
        .await?;

    loop {
        async_std::task::sleep(Duration::from_secs(5)).await;
        event!(Level::INFO, "An message at info level");
        async_std::task::sleep(Duration::from_secs(1)).await;
        event!(Level::WARN, "An message at warning level");
    }
}
```

See [tracing-server.rs](https://github.com/swsnr/logcontrol.rs/blob/main/zbus/examples/tracing-server.rs) for a more complete example.
