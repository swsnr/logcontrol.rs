# logcontrol.rs

Types and implementations for systemd's [logcontrol] interface.

This interface provides means to change logging behaviour of system services at runtime, over D-Bus, or via `systemctl service-log-level` or `systemctl service-log-target`.

This repository provides a collection of traits of basic types and implementations of this interface:

- `logcontrol` contains the basic types and defines an abstract trait for the interface.
- [`logcontrol-tracing`](./tracing) provides a logcontrol backend implementation for the [`tracing`][tracing] library.
- [`logcontrol-zbus`](./zbus) provides a DBus interface implementation for [`zbus`][zbus] DBus framework.

[logcontrol]: https://www.freedesktop.org/software/systemd/man/org.freedesktop.LogControl1.html#
[tracing]: https://github.com/tokio-rs/tracing
[zbus]: https://github.com/dbus2/zbus
