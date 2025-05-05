# logcontrol-zbus

[![Crates.io](https://img.shields.io/crates/v/logcontrol-zbus)](https://crates.io/crates/logcontrol-zbus)
[![docs.rs](https://img.shields.io/docsrs/logcontrol-zbus)](https://docs.rs/logcontrol-zbus)

[`zbus`][zbus] DBus frontend for the [logcontrol] interface.

[zbus]: https://github.com/dbus2/zbus
[logcontrol]: https://codeberg.org/swsnr/logcontrol.rs

## Usage

```console
$ cargo add logcontrol-zbus
```

```rust
use logcontrol_zbus::ConnectionBuilderExt;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use an implementation such as logcontrol-tracing
    let control = create_log_control();
    let _conn = zbus::ConnectionBuilder::session()?
        .name("de.swsnr.logcontrol.SimpleServerExample")?
        .serve_log_control(logcontrol_zbus::LogControl1::new(control))?
        .build()
        .await?;

    // Do other things or go to wait forever
    std::future::pending::<()>().await;

    Ok(())
}
```
