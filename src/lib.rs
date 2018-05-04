//! A Rust client for interacting with Dogstatsd
//!
//! Dogstatsd is a custom `StatsD` implementation by [Datadog](https://datadog.com) for sending metrics
//! and events to their system. Through this client you can report any type of metric you want, tag it,
//! and enjoy your custom metrics.
//!
//! ## Usage
//!
//! Build an options struct and create a client:
//!
//! ```
//! use dogstatsd::{Client, Options};
//!
//! // Binds to a udp socket on an available ephemeral port on 127.0.0.1 for
//! // transmitting, and sends to  127.0.0.1:8125, the default dogstatsd
//! // address.
//! let default_client = Client::new();
//!
//! // Binds to 127.0.0.1:9000 for transmitting and sends to 10.1.2.3:8125, with a
//! // namespace of "analytics".
//! let custom_options = Options { from_addr: "127.0.0.1:9000", to_addr: "10.1.2.3:8125", namespace: "analytics" };
//! let custom_client = Client::new_with_options(custom_options).unwrap();
//! ```
//!
//! Start sending metrics:
//!
//! ```
//! use dogstatsd::Client;
//!
//! let client = Client::new().unwrap();
//! let tags = &["env:production"];
//!
//! // Increment a counter
//! client.incr("my_counter", tags).unwrap();
//!
//! // Decrement a counter
//! client.decr("my_counter", tags).unwrap();
//!
//! // Time a block of code (reports in ms)
//! client.time("my_time", tags, || {
//!     // Some time consuming code
//! }).unwrap();
//!
//! // Report your own timing in ms
//! client.timing("my_timing", 500, tags).unwrap();
//!
//! // Report an arbitrary value (a gauge)
//! client.gauge("my_gauge", 1.2345, tags).unwrap();
//!
//! // Report a sample of a histogram
//! client.histogram("my_histogram", 67890, tags).unwrap();
//!
//! // Report a sample of a distribution
//! client.distribution("distribution", 67890, tags).unwrap();
//!
//! // Report a member of a set
//! client.set("my_set", "13579", tags).unwrap();
//!
//! // Send a custom event
//! client.event("My Custom Event Title", "My Custom Event Body", tags).unwrap();
//! ```

#![cfg_attr(feature = "unstable", feature(test))]
#![deny(warnings, missing_debug_implementations, missing_copy_implementations, missing_docs)]

extern crate chrono;
extern crate rand;

use chrono::Utc;
use std::net::UdpSocket;

use rand::Rng;

mod error;
pub use self::error::DogstatsdError;

mod metrics;
use self::metrics::*;
pub use self::metrics::{ServiceStatus, ServiceCheckOptions};

/// A type alias for returning a unit type or an error
pub type DogstatsdResult = Result<(), DogstatsdError>;

/// The struct that represents the options available for the Dogstatsd client.
#[derive(Debug, PartialEq)]
pub struct Options<'a> {
    /// The address of the udp socket we'll bind to for sending.
    pub from_addr: &'a str,
    /// The address of the udp socket we'll send metrics and events to.
    pub to_addr: &'a str,
    /// A namespace to prefix all metrics with, joined with a '.'.
    pub namespace: &'a str,
}

impl<'a> Default for Options<'a> {
    /// Create a new options struct with all the default settings.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Options;
    ///
    ///   let options = Options::default();
    ///
    ///   assert_eq!(
    ///       Options {
    ///           from_addr: "127.0.0.1:0",
    ///           to_addr: "127.0.0.1:8125",
    ///           namespace: "",
    ///       },
    ///       options
    ///   )
    /// ```
    fn default() -> Self {
        Options {
            from_addr: "127.0.0.1:0",
            to_addr: "127.0.0.1:8125",
            namespace: "",
        }
    }
}

/// The client struct that handles sending metrics to the Dogstatsd server.
#[derive(Debug)]
pub struct Client<'a> {
    socket: UdpSocket,
    options: Options<'a>,
    rng: rand::XorShiftRng,
}

impl<'a> PartialEq for Client<'a> {
    fn eq(&self, other: &Self) -> bool {
        // Ignore `socket`, which will never be the same
        self.options == other.options
    }
}

impl<'c> Client<'c> {
    /// Create a new client from an options struct.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    /// ```
    pub fn new() -> Result<Self, DogstatsdError> {
        Client::new_with_options(Options::default())
    }

    /// Create a new client from an options struct.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new_with_options(Options::default()).unwrap();
    /// ```
    pub fn new_with_options(options: Options<'c>) -> Result<Self, DogstatsdError> {
        Ok(Client {
            socket: UdpSocket::bind(&options.from_addr)?,
            options: options,
            rng: rand::weak_rng(),
        })
    }

    /// Increment a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.incr("counter", &["tag:counter"]).unwrap();
    /// ```
    pub fn incr<I>(&self, stat: &str, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        self.count(stat, 1, tags)
    }

    /// Increment a StatsD counter with rate
    pub fn rated_incr<I>(&mut self, stat: &str, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        if self.rng.next_f32() < rate {
            self.incr(stat, tags)
        } else {
            Ok(())
        }
    }

    /// Decrement a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.decr("counter", &["tag:counter"]).unwrap();
    /// ```
    pub fn decr<I>(&self, stat: &str, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        self.count(stat, -1, tags)
    }

    /// Rated decrement a StatsD counter
    pub fn rated_decr<I>(&mut self, stat: &str, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        if self.rng.next_f32() < rate {
            self.decr(stat, tags)
        } else {
            Ok(())
        }
    }

    /// Measure a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.count("counter", 10, &["tag:counter"]).unwrap();
    /// ```
    pub fn count<I, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<i64>
    {
        self.send(&Metric::Count { stat, val: val.into() }, tags)
    }

    /// Rated measure a StatsD counter
    pub fn rated_count<I, V>(&mut self, stat: &str, val: V, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<i64>
    {
        if self.rng.next_f32() < rate {
            self.count(stat, val, tags)
        } else {
            Ok(())
        }
    }

    /// Time how long it takes for a block of code to execute.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///   use std::thread;
    ///   use std::time::Duration;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.time("timer", &["tag:time"], || {
    ///       thread::sleep(Duration::from_millis(200))
    ///   }).unwrap();
    /// ```
    pub fn time<I, F>(&self, stat: &str, tags: I, block: F) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              F: FnOnce()
    {
        let start_time = Utc::now();
        block();
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);
        self.send(&Metric::Timing { stat, val: duration.into() }, tags)
    }

    /// Send timing metric in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    ///   # extern crate chrono;
    ///   # extern crate dogstatsd;
    ///   # fn main() {
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.timing("timing", 350, &["tag:timing"]).unwrap();
    ///   client.timing("timing", chrono::Duration::seconds(2), &["tag:timing"]).unwrap();
    ///   # }
    /// ```
    pub fn timing<I, V>(&self, stat: &str, duration: V, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<DurationMeasurement>
    {
        self.send(&Metric::Timing { stat, val: duration.into() }, tags)
    }

    /// Rated timing metric in milliseconds
    pub fn rated_timing<I, V>(&mut self, stat: &str, duration: V, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<DurationMeasurement>
    {
        if self.rng.next_f32() < rate {
            self.timing(stat, duration, tags)
        } else {
            Ok(())
        }
    }

    /// Report gauge measurement
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.gauge("gauge", 12345, &["tag:gauge"]).unwrap();
    ///   client.gauge("gauge", 123.45, &["tag:gauge"]).unwrap();
    /// ```
    pub fn gauge<I, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<Measurement>
    {
        self.send(&Metric::Gauge { stat, val: val.into() }, tags)
    }

    /// Rated gauge measurement
    pub fn rated_gauge<I, V>(&mut self, stat: &str, val: V, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<Measurement>
    {
        if self.rng.next_f32() < rate {
            self.gauge(stat, val, tags)
        } else {
            Ok(())
        }
    }

    /// Report a value in a histogram
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.histogram("histogram", 67.890, &["tag:histogram"]).unwrap();
    /// ```
    pub fn histogram<I, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<Measurement>
    {
        self.send(&Metric::Histogram { stat, val: val.into() }, tags)
    }

    /// Rated histogram measurement
    pub fn rated_histogram<I, V>(&mut self, stat: &str, val: V, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<Measurement>
    {
        if self.rng.next_f32() < rate {
            self.histogram(stat, val, tags)
        } else {
            Ok(())
        }
    }

    /// Report a value in a distribution
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.distribution("distribution", 67890, &["tag:distribution"]).unwrap();
    ///   client.distribution("distribution", 67.890, &["tag:distribution"]).unwrap();
    /// ```
    pub fn distribution<I, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<Measurement>
    {
        self.send(&Metric::Distribution { stat, val: val.into() }, tags)
    }

    /// Rated distribution measurement
    pub fn rated_distribution<I, V>(&mut self, stat: &str, val: V, tags: I, rate: f32) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<Measurement>
    {
        if self.rng.next_f32() < rate {
            self.distribution(stat, val, tags)
        } else {
            Ok(())
        }
    }

    /// Report a value in a set
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.set("set", "13579", &["tag:set"]).unwrap();
    /// ```
    pub fn set<'a, I, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>,
              V: Into<SetMeasurement<'a>>
    {
        self.send(&Metric::Set { stat, val: val.into() }, tags)
    }

    /// Report the status of a service
    ///
    /// # Examples
    ///
    /// ```
    ///   # extern crate chrono;
    ///   # extern crate dogstatsd;
    ///   # fn main() {
    ///   use dogstatsd::{Client, ServiceStatus, ServiceCheckOptions};
    ///
    ///   let client = Client::new().unwrap();
    ///   let tags = &["tag:service"];
    ///   
    ///   client.service_check("redis.can_connect", ServiceStatus::OK, tags, None).unwrap();
    ///
    ///   client.service_check("redis.can_connect", ServiceStatus::OK, tags, 
    ///     Some(ServiceCheckOptions {
    ///       hostname: Some("my-host.localhost"),
    ///       ..Default::default()
    ///     })
    ///   ).unwrap();
    ///
    ///   client.service_check("redis.can_connect", ServiceStatus::OK, tags, 
    ///     Some(ServiceCheckOptions {
    ///       hostname: Some("my-host.localhost"),
    ///       timestamp: Some(chrono::Utc::now().into()),
    ///       message: Some("Message about the check or service")
    ///     })
    ///   ).unwrap();
    ///   # }
    /// ```
    pub fn service_check<I>(&self, stat: &str, val: ServiceStatus, tags: I, options: Option<ServiceCheckOptions>) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        let opt = options.unwrap_or(ServiceCheckOptions::default());
        self.send(&Metric::ServiceCheck { stat, val, opt }, tags)
    }

    /// Send a custom event as a title and a body
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Client;
    ///
    ///   let client = Client::new().unwrap();
    ///   client.event("Event Title", "Event Body", &["tag:event"]).unwrap();
    /// ```
    pub fn event<I>(&self, title: &str, text: &str, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        self.send(&Metric::Event { title, text }, tags)
    }

    fn send<I>(&self, metric: &Metric, tags: I) -> DogstatsdResult
        where I: IntoIterator, I::Item: AsRef<str>
    {
        let formatted_metric = format_for_send(metric, &self.options.namespace, tags);
        self.socket.send_to(formatted_metric.as_slice(), &self.options.to_addr)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let options = Options::default();
        let expected_options = Options {
            from_addr: "127.0.0.1:0",
            to_addr: "127.0.0.1:8125",
            namespace: "",
        };

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_new() {
        let client = Client::new().unwrap();
        let expected_client = Client {
            socket: UdpSocket::bind("127.0.0.1:0").unwrap(),
            options: Options {
                from_addr: "127.0.0.1:0",
                to_addr: "127.0.0.1:8125",
                namespace: "",
            },
            rng: rand::weak_rng(),
        };

        assert_eq!(expected_client, client)
    }

    #[test]
    fn test_send() {
        let options = Options { from_addr: "127.0.0.1:9001", to_addr: "127.0.0.1:9002", namespace: "" };
        let client = Client::new_with_options(options).unwrap();
        // Shouldn't panic or error
        client.send(&Metric::Gauge { stat: "gauge", val: 1234.into() }, &["tag1", "tag2"]).unwrap();
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;
    use self::test::Bencher;
    use super::*;

    #[bench]
    fn bench_incr(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = &["name1:value1"];
        b.iter(|| {
            client.incr("bench.incr", tags).unwrap();
        })
    }

    #[bench]
    fn bench_decr(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = &["name1:value1"];
        b.iter(|| {
            client.decr("bench.decr", tags).unwrap();
        })
    }

    #[bench]
    fn bench_timing(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = &["name1:value1"];
        let mut i = 0;
        b.iter(|| {
            client.timing("bench.timing", i, tags).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_gauge(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        let mut i = 0;
        b.iter(|| {
            client.gauge("bench.guage", &i.to_string(), &tags).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_histogram(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        let mut i = 0;
        b.iter(|| {
            client.histogram("bench.histogram", &i.to_string(), &tags).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_distribution(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        let mut i = 0;
        b.iter(|| {
            client.distribution("bench.distribution", &i.to_string(), &tags).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_set(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        let mut i = 0;
        b.iter(|| {
            client.set("bench.set", &i.to_string(), &tags).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_service_check(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        b.iter(|| {
            client.service_check("bench.service_check", ServiceStatus::Critical, &tags).unwrap();
        })
    }

    #[bench]
    fn bench_event(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        b.iter(|| {
            client.event("Test Event Title", "Test Event Message", &tags).unwrap();
        })
    }
}
