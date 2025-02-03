# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.2] – 2025-02-03

### Changed
- Add `must_use` to some functions.

### Removed
- Drop `thiserror` dependency.

## [1.0.1] – 2023-09-30

### Fixed
- Fix documentation typos and crate metadata.

## [1.0.0] – 2023-09-30

Initial release.

### Added

- Types and trait for the log control interface: `logcontrol::LogLevel`, `logcontrol::KnownLogTarget`, and `logcontrol::LogControl1`
- Helper functions:
    - `DBUS_OBJ_PATH`
    - `stderr_connected_to_journal`
    - `syslog_identifier`

[Unreleased]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-v1.0.2...HEAD
[1.0.2]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-v1.0.1...logcontrol-v1.0.2
[1.0.1]: https://github.com/swsnr/logcontrol.rs/compare/logcontrol-v1.0.0...logcontrol-v1.0.1
[1.0.0]: https://github.com/swsnr/logcontrol.rs/releases/tag/logcontrol-v1.0.0
