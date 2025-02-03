# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Bump MSRV to 1.77.

## [3.0.1] – 2025-02-03

### Changed
- Improve documentation.

## [3.0.0] – 2024-11-01

### Changed
- Update zbus to version 5.

## [2.0.0] – 2024-02-13

### Changed
- Update zbus to version 4 (see [GH-3]).

[GH-3]: https://github.com/swsnr/logcontrol.rs/pull/3

## [1.1.0] – 2023-09-30

### Added
- Re-export `logcontrol` and `logcontrol::DBUS_OBJ_PATH`.
- Add `logcontrol_zbus::ConnectionBuilderExt` to extend `ConnectionBuilder` with `serve_log_control`.

## [1.0.0] – 2023-09-30

Initial release.

### Added

- Add DBus interface implementation `LogControl1`.

[Unreleased]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-zbus-v3.0.1...HEAD
[3.0.1]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-zbus-v3.0.0...logcontrol-zbus-v3.0.1
[3.0.0]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-zbus-v2.0.0...logcontrol-zbus-v3.0.0
[2.0.0]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-zbus-v1.1.0...logcontrol-zbus-v2.0.0
[1.1.0]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-zbus-v1.0.0...logcontrol-zbus-v1.1.0
[1.0.0]: https://github.com/swsnr/logcontrol.rs/releases/tag/logcontrol-zbus-v1.0.0
