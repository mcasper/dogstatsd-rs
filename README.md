dogstatsd-rs
============
[![Build Status](https://travis-ci.org/mcasper/dogstatsd-rs.svg?branch=master)](https://travis-ci.org/mcasper/dogstatsd-rs)
[![Crate Version](https://img.shields.io/crates/v/dogstatsd.svg)](https://crates.io/crates/dogstatsd)

A Rust client for interacting with Dogstatsd

Dogstatsd is a custom StatsD implementation by DataDog for sending metrics and
events to their system. Through this client you can report any type of metric
you want, tag it, and enjoy your custom metrics.

[Full Documentation](https://mcasper.github.io/dogstatsd-rs/dogstatsd/)

## Usage

Build an options struct and create a client:
```rust
use dogstatsd::{Client, Options};

// Binds to a udp socket on an available ephemeral port on 127.0.0.1 for
// transmitting, and sends to 127.0.0.1:8125, the default dogstatsd address.
let default_options = Options::default();
let default_client = Client::new(default_options).unwrap();

// Binds to 127.0.0.1:9000 for transmitting and sends to 10.1.2.3:8125, with a
// namespace of "analytics".
let custom_options = Options::new("127.0.0.1:9000", "10.1.2.3:8125", "analytics");
let custom_client = Client::new(custom_options).unwrap();
```

Start sending metrics:
```rust
use dogstatsd::{Client, Options, ServiceCheckOptions, ServiceStatus};

let client = Client::new(Options::default()).unwrap();
let tags = &["env:production"];

// Increment a counter
client.incr("my_counter", tags).unwrap();

// Decrement a counter
client.decr("my_counter", tags).unwrap();

// Time a block of code (reports in ms)
client.time("my_time", tags, || {
    // Some time consuming code
}).unwrap();

// Report your own timing in ms
client.timing("my_timing", 500, tags).unwrap();

// Report an arbitrary value (a gauge)
client.gauge("my_gauge", "12345", tags).unwrap();

// Report a sample of a histogram
client.histogram("my_histogram", "67890", tags).unwrap();

// Report a sample of a distribution
client.distribution("distribution", "67890", tags).unwrap();

// Report a member of a set
client.set("my_set", "13579", tags).unwrap();

// Report a service check
let service_check_options = ServiceCheckOptions {
  hostname: Some("my-host.localhost"),
  ..Default::default()
};
client.service_check("redis.can_connect", ServiceStatus::OK, tags, Some(service_check_options)).unwrap();

// Send a custom event
client.event("My Custom Event Title", "My Custom Event Body", tags).unwrap();
```

## Async

To use the `tokio` client for asynchronous emission, enable the `tokio` feature of this library. Usage should look almost identical to the synchronous interface.

```rust
let options = Options::new("127.0.0.1:9001", "127.0.0.1:9002", "");
let client = Client::new(options).await.unwrap();
// Shouldn't panic or error
client.send(&GaugeMetric::new("gauge".into(), "1234".into()), &["tag1", "tag2"]).await.unwrap();
```

## Benchmarks

Support is provided for running benchmarks of all client commands. Until the
`Bencher` type is stable Rust, the benchmarks are isolated behind the
`unstable` feature flag. To run the benchmarks using `rustup`:

```
    rustup run nightly cargo bench --features=unstable
```
