# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic
Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2018-04-26

### Added
- Support for the distribution metric type - https://github.com/mcasper/dogstatsd-rs/commit/e04d0ee913da93c91d9f41c94dd9d3b099b511a7
- Make DogstatsdError type public - https://github.com/mcasper/dogstatsd-rs/pull/21<Paste>

### Improved
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
