# Changelog

This changelog tries to adhere to the [Common Changelog](https://common-changelog.org)
specification.


## [0.3.0] - 2026-03-13

A switch to use JSON as the configuration file format.

### Added

 - New configuration options in the local configuration (`autocommit`, `autoignore`)
 - Support for --version or -v parameter to print the version
 - A test harness and some basic integration tests

### Changed

 - Switch to use JSON configuration files
 - Print all tables in a consistent way as Unicode tables

### Removed

 - Support for YAML configuration files


## [0.2.0] - 2026-03-10

Make it work on Windows.

### Added

  - Windows support
  - New command: `repo server edit`

### Changed

  - Replaced `env_logger` with `tracing`


## [0.1.0] - 2026-03-09

A rewrite of the old Python tool in Rust with some added features.
