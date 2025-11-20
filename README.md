# logcontrol.rs

[![Crates.io](https://img.shields.io/crates/v/logcontrol)](https://crates.io/crates/logcontrol)
[![docs.rs](https://img.shields.io/docsrs/logcontrol)](https://docs.rs/logcontrol)

Types and implementations for systemd's [logcontrol] interface.

This interface provides means to change logging behaviour of system services at runtime, over D-Bus, or via `systemctl service-log-level` or `systemctl service-log-target`.

This repository provides a collection of traits of basic types and implementations of this interface:

- `logcontrol` contains the basic types and defines an abstract trait for the interface.
- [`logcontrol-tracing`](https://codeberg.org/swsnr/logcontrol.rs/src/branch/main/tracing) provides a logcontrol backend implementation for the [`tracing`][tracing] library.
- [`logcontrol-log`](https://codeberg.org/swsnr/logcontrol.rs/src/branch/main/log) provides a logcontrol backend implementation for the [`log`][log] library.
- [`logcontrol-zbus`](https://codeberg.org/swsnr/logcontrol.rs/src/branch/main/zbus) provides a DBus interface implementation for [`zbus`][zbus] DBus framework.

[logcontrol]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html#
[tracing]: https://github.com/tokio-rs/tracing
[log]: https://github.com/rust-lang/log
[zbus]: https://github.com/dbus2/zbus

## Usage

```console
$ cargo add logcontrol-tracing
$ cargo add logcontrol-zbus
```

See [`tracing/examples/zbus_tracing.rs`](./tracing/examples/zbus_tracing.rs) for a complete example with [zbus] and [tracing],
and [`log/examples/zbus_log.rs`](./log/examples/zbus_log.rs) for an example using the [log] crate.
