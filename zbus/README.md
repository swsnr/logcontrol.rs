# logcontrol-zbus

[`zbus`][zbus] DBus frontend for the [logcontrol] interface.

[zbus]: https://github.com/dbus2/zbus
[logcontrol]: https://github.com/swsnr/logcontrol.rs

## Usage

```console
$ cargo add logcontrol-zbus
```

```rust
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use an implementation such as logcontrol-tracing
    let control = create_log_control();
    let _conn = zbus::ConnectionBuilder::session()?
        .name("de.swsnr.logcontrol.SimpleServerExample")?
        .serve_at(
            logcontrol::DBUS_OBJ_PATH,
            logcontrol_zbus::LogControl1::new(control),
        )?
        .build()
        .await?;

    // Do other things or go to wait forever
    std::future::pending::<()>().await;

    Ok(())
}
```
