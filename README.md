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

// Binds to a udp socket on 127.0.0.1:8126 for transmitting, and sends to
// 127.0.0.1:8125, the default dogstatsd address.
let default_options = Options::default();
let default_client = Client::new(default_options);

// Binds to 127.0.0.1:9000 for transmitting and sends to 10.1.2.3:8125, with a
// namespace of "analytics".
let custom_options = Options::new("127.0.0.1:9000", "10.1.2.3:8125", "analytics");
let custom_client = Client::new(custom_options);
```

Start sending metrics:
```rust
use dogstatsd::{Client, Options};

let client = Client::new(Options::default());

// Increment a counter
client.incr("my_counter", vec![]).unwrap();

// Decrement a counter
client.decr("my_counter", vec![]).unwrap();

// Time a block of code (reports in ms)
client.time("my_time", vec![], || {
    // Some time consuming code
}).unwrap();

// Report your own timing in ms
client.timing("my_timing", 500, vec![]).unwrap();

// Report an arbitrary value (a gauge)
client.gauge("my_gauge", "12345", vec![]).unwrap();

// Report a sample of a histogram
client.histogram("my_histogram", "67890", vec![]).unwrap();

// Report a member of a set
client.set("my_set", "13579", vec![]).unwrap();

// Send a custom event
client.event("My Custom Event Title", "My Custom Event Body", vec![]).unwrap();

// Add tags to any metric by passing a Vec<String> of tags to apply
client.gauge("my_gauge", "12345", vec!["tag:1".into(), "tag:2".into()]).unwrap();
```
