# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.11.1] - 2023-11-22

### Changed

Don't require static strings for ServiceCheckOptions - https://github.com/mcasper/dogstatsd-rs/pull/65

## [0.11.0] - 2023-09-27

### Added

- Support standard system tags from DD_* environment variables - https://github.com/mcasper/dogstatsd-rs/pull/61

### Changed

- Improved UDS support - https://github.com/mcasper/dogstatsd-rs/pull/62

## [0.10.0] - 2023-08-24

### Added

- Support for batching metrics - https://github.com/mcasper/dogstatsd-rs/pull/60

## [0.9.0] - 2023-07-27

### Added

- Support for incrementing/decrementing counters by a value other than 1 - https://github.com/mcasper/dogstatsd-rs/pull/50

## [0.8.1] - 2023-05-03

### Changed

- The `time` and `async_time` functions now return the output in the event of an error - https://github.com/mcasper/dogstatsd-rs/pull/47
- The default from address is now 0.0.0.0 - https://github.com/mcasper/dogstatsd-rs/pull/48

## [0.8.0] - 2023-03-28

### Added

- Support for timing async functions - https://github.com/mcasper/dogstatsd-rs/pull/45
- Support for returning a value from timed functions - https://github.com/mcasper/dogstatsd-rs/pull/42

## [0.7.1] - 2022-07-26

### Removed

- Remove dependency on time crate - https://github.com/mcasper/dogstatsd-rs/pull/36

## [0.7.0] - 2022-04-25

### Added

- OptionsBuilder API for more flexibly specifying client options - https://github.com/mcasper/dogstatsd-rs/pull/35
- Ability to provide default tags that should be sent with all events - https://github.com/mcasper/dogstatsd-rs/pull/34

## [0.6.2] - 2021-01-26

### Fixed

- Build error from deprecated `Error::description` impl, replaced with `Error::source` - https://github.com/mcasper/dogstatsd-rs/pull/31

## [0.6.1] - 2019-03-16

### Fixed

- Segfault in DogstatsdError Display implementation - https://github.com/mcasper/dogstatsd-rs/pull/25

## [0.6.0] - 2019-02-23

### Added

- Support for an arbitrary count metric, instead of just incrementing or decrementing - https://github.com/mcasper/dogstatsd-rs/pull/24

### Fixed

- Benchmarks compile and run again - https://github.com/mcasper/dogstatsd-rs/pull/24
- Decrement metric string allocation - https://github.com/mcasper/dogstatsd-rs/pull/24

## [0.5.0] - 2018-04-26

### Added

- Support for the distribution metric type - https://github.com/mcasper/dogstatsd-rs/commit/e04d0ee913da93c91d9f41c94dd9d3b099b511a7
- Make DogstatsdError type public - https://github.com/mcasper/dogstatsd-rs/pull/21

### Changed

- Allow stat and value arguments to be separate types - https://github.com/mcasper/dogstatsd-rs/commit/f1927ad1918f821fc6771a20b2b9c4abf9c0bcd9

## [0.4.1] - 2018-03-02

### Fixed

- Don't send namespaces for service checks or events - https://github.com/mcasper/dogstatsd-rs/pull/18

## [0.4] - 2017-11-10

### Added

- Support service checks
- Bump our Chrono version to 0.4, update use of their API

## [0.3.1] - 2017-11-08

### Added

- Implement `Default` for `Options`

## [0.3] - 2017-04-21

### Added

- Use IntoIter for tags instead of a specific type
- Use String builder methods to make metric building more efficient - https://github.com/mcasper/dogstatsd-rs/pull/13
- Allow client methods to take `&str` and `&[]` as well - https://github.com/mcasper/dogstatsd-rs/pull/13

## [0.2] - 2017-04-12

### Added

- Add benchmarks for all the client commands - https://github.com/mcasper/dogstatsd-rs/pull/10
- Use an automatically assigned ephemeral port for sending - https://github.com/mcasper/dogstatsd-rs/pull/11
- Reuse the same UDP socket between metrics from the same client - https://github.com/mcasper/dogstatsd-rs/pull/12

## [0.1.1] - 2016-07-07

### Fixed

- Fix tag formatting

## [0.1] - 2016-04-26

### Added

- Initial Release
