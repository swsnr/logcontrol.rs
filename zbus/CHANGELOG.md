# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Re-export `logcontrol` and `logcontrol::DBUS_OBJ_PATH`.
- Add `logcontrol_zbus::ConnectionBuilderExt` to extend `ConnectionBuilder` with `serve_log_control`.

## [1.0.0] – 2023-09-30

Initial release.

### Added

- Add DBus interface implementation `LogControl1`.

[Unreleased]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-zbus-v1.0.0...HEAD
[1.0.0]: https://github.com/swsnr/logcontrol.rs/releases/tag/logcontrol-zbus-v1.0.0
