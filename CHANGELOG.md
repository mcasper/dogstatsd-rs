## 0.4.1

* Don't send namespaces for service checks or events - https://github.com/mcasper/dogstatsd-rs/pull/18

## 0.4

* Support service checks
* Bump our Chrono version to 0.4, update use of their API

## 0.3.1

* Implement `Default` for `Options`

## 0.3

* Use IntoIter for tags instead of a specific type
* Use String builder methods to make metric building more efficient - https://github.com/mcasper/dogstatsd-rs/pull/13
* Allow client methods to take `&str` and `&[]` as well - https://github.com/mcasper/dogstatsd-rs/pull/13

## 0.2

* Add benchmarks for all the client commands - https://github.com/mcasper/dogstatsd-rs/pull/10
* Use an automatically assigned ephemeral port for sending - https://github.com/mcasper/dogstatsd-rs/pull/11
* Reuse the same UDP socket between metrics from the same client - https://github.com/mcasper/dogstatsd-rs/pull/12

## 0.1.1

* Fix tag formatting

## 0.1

* Initial Release
