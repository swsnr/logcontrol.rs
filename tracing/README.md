# logcontrol-tracing

[![Crates.io](https://img.shields.io/crates/v/logcontrol-tracing)](https://crates.io/crates/logcontrol-tracing)
[![docs.rs](https://img.shields.io/docsrs/logcontrol-tracing)](https://docs.rs/logcontrol-tracing)

[`tracing`][tracing] implementation for the [logcontrol] interface.

[tracing]: https://github.com/tokio-rs/tracing
[logcontrol]: https://github.com/swsnr/logcontrol.rs

## Usage

```console
$ cargo add logcontrol-tracing
```

```rust
use logcontrol::*;
use logcontrol_tracing::*;
use tracing_subscriber::prelude::*;

let (control, layer) = TracingLogControl1::new_auto(
    PrettyLogControl1LayerFactory,
    tracing::Level::INFO,
).unwrap();

let subscriber = tracing_subscriber::Registry::default().with(layer);
tracing::subscriber::set_global_default(subscriber).unwrap();
// Then register `control` over DBus, e.g. via `logcontrol_zbus::LogControl1`.
```
