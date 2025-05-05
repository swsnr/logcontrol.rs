# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] – 2025-05-05

### Changed
- Bump MSRV to 1.77.

## [0.2.1] – 2025-02-03

### Changed
- Add missing `fmt` and `ansi` features to `tracing-subscriber` dependency.

## [0.2.0] – 2023-09-30

### Added
- Re-export `logcontrol`, `logcontrol::syslog_identifier` and `logcontrol::stderr_connected_to_journal`.

## [0.1.0] – 2023-09-30

Initial release.

### Added

- Factory types for layers: `LogControl1LayerFactory` and `PrettyLogControl1LayerFactory`.
- Log control implementation: `TracingLogControl1`

[Unreleased]: https://codeberg.org/swsnr/logcontrol.rs/compare/logcontrol-tracing-v0.2.2...HEAD
[0.2.2]: https://codeberg.org/swsnr/logcontrol.rs/compare/logcontrol-tracing-v0.2.1...logcontrol-tracing-v0.2.2
[0.2.1]: https://codeberg.org/swsnr/logcontrol.rs/compare/logcontrol-tracing-v0.2.0...logcontrol-tracing-v0.2.1
[0.2.0]: https://codeberg.org/swsnr/logcontrol.rs/compare/logcontrol-tracing-v0.1.0...logcontrol-tracing-v0.2.0
[0.1.0]: https://codeberg.org/swsnr/logcontrol.rs/releases/tag/logcontrol-zbus-v1.0.0
