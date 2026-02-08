# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.4] - 2026-02-08
### Added
- New `UrlOrPath` helper type

## [0.3.3] - 2026-02-08
### Added
- New `Config::with_sources()` helper

## [0.3.2] - 2026-02-08
### Changed
- Improved error trait bounds in `Format` and `Source`

## [0.3.1] - 2026-02-08
### Changed
- Improved `Format` trait

## [0.3.0] - 2026-02-07
### Changed
- Removed dependency on `figment2`
### Fixed
- Allow mixed unit + unnamed variant enums - those will be parsed as tagged

## [0.2.0] - 2026-02-04
### Added
- Applying `#[serde_with::skip_serializing_none]`
- Macro hygiene
- Proc `derive` macro for deduplication of explicit derives (e.g. `#[derive(setty::Config, serde::Serialize)`)
- Parsing facade
- JSON Schema output
- Markdown output
- Path completions
- Print out (with / without defaults)
- Case variations for struct fields and enum variants
- Extended types support (`chrono`, `bigdecimal`, `duration-string`)
- Per-field combine strategies

## [0.1.0] - 2026-01-24
### Added
- Keeping a CHANGELOG
- Initial version
