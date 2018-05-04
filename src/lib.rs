//! A Rust client for interacting with Dogstatsd
//!
//! Dogstatsd is a custom `StatsD` implementation by `DataDog` for sending metrics and events to their
//! system. Through this client you can report any type of metric you want, tag it, and enjoy your
//! custom metrics.
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
//! let default_options = Options::default();
//! let default_client = Client::new(default_options).unwrap();
//!
//! // Binds to 127.0.0.1:9000 for transmitting and sends to 10.1.2.3:8125, with a
//! // namespace of "analytics".
//! let custom_options = Options::new("127.0.0.1:9000", "10.1.2.3:8125", "analytics");
//! let custom_client = Client::new(custom_options).unwrap();
//! ```
//!
//! Start sending metrics:
//!
//! ```
//! use dogstatsd::{Client, Options};
//!
//! let client = Client::new(Options::default()).unwrap();
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
//! client.gauge("my_gauge", 12.345, tags).unwrap();
//!
//! // Report a sample of a histogram
//! client.histogram("my_histogram", 67890, tags).unwrap();
//!
//! // Report a sample of a distribution
//! client.distribution("distribution", 67890, tags).unwrap();
//!
//! // Report a member of a set
//! client.set("my_set", 13579, tags).unwrap();
//!
//! // Send a custom event
//! client.event("My Custom Event Title", "My Custom Event Body", tags).unwrap();
//! ```

#![cfg_attr(feature = "unstable", feature(test))]
#![deny(warnings, missing_debug_implementations, missing_copy_implementations, missing_docs)]
extern crate chrono;

use chrono::Utc;
use std::net::UdpSocket;

mod error;
pub use self::error::DogstatsdError;

mod metrics;
use self::metrics::*;
pub use self::metrics::{ServiceStatus, ServiceCheckOptions};

/// A type alias for returning a unit type or an error
pub type DogstatsdResult = Result<(), DogstatsdError>;

/// The struct that represents the options available for the Dogstatsd client.
#[derive(Debug, PartialEq)]
pub struct Options {
    /// The address of the udp socket we'll bind to for sending.
    pub from_addr: String,
    /// The address of the udp socket we'll send metrics and events to.
    pub to_addr: String,
    /// A namespace to prefix all metrics with, joined with a '.'.
    pub namespace: String,
}

impl Default for Options {
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
    ///           from_addr: "127.0.0.1:0".into(),
    ///           to_addr: "127.0.0.1:8125".into(),
    ///           namespace: String::new(),
    ///       },
    ///       options
    ///   )
    /// ```
    fn default() -> Self {
        Options {
            from_addr: "127.0.0.1:0".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        }
    }

}

impl Options {
    /// Create a new options struct by supplying values for all fields.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::Options;
    ///
    ///   let options = Options::new("127.0.0.1:9000", "127.0.0.1:9001", "");
    /// ```
    pub fn new(from_addr: &str, to_addr: &str, namespace: &str) -> Self {
        Options {
            from_addr: from_addr.into(),
            to_addr: to_addr.into(),
            namespace: namespace.into(),
        }
    }
}

/// The client struct that handles sending metrics to the Dogstatsd server.
#[derive(Debug)]
pub struct Client {
    socket: UdpSocket,
    from_addr: String,
    to_addr: String,
    namespace: String,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        // Ignore `socket`, which will never be the same
        self.from_addr == other.from_addr &&
        self.to_addr == other.to_addr &&
        self.namespace == other.namespace
    }
}

impl Client {
    /// Create a new client from an options struct.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    /// ```
    pub fn new(options: Options) -> Result<Self, DogstatsdError> {
        Ok(Client {
            socket: UdpSocket::bind(&options.from_addr)?,
            from_addr: options.from_addr,
            to_addr: options.to_addr,
            namespace: options.namespace,
        })
    }

    /// Increment a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.incr("counter", &["tag:counter"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn incr<I, T>(&self, stat: &str, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
    {
        let val = 1;
        self.send(&Metric::Count { stat, val: val.into() }, tags)
    }

    /// Decrement a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.decr("counter", &["tag:counter"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn decr<I, T>(&self, stat: &str, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
    {
        let val = -1;
        self.send(&Metric::Count { stat, val: val.into() }, tags)
    }

    /// Time how long it takes for a block of code to execute.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///   use std::thread;
    ///   use std::time::Duration;
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.time("timer", &["tag:time"], || {
    ///       thread::sleep(Duration::from_millis(200))
    ///   }).unwrap_or_else(|e| println!("Encountered error: {}", e))
    /// ```
    pub fn time<F, I, T>(&self, stat: &str, tags: I, block: F) -> DogstatsdResult
        where F: FnOnce(),
              I: IntoIterator<Item=T>,
              T: AsRef<str>,
    {
        let start_time = Utc::now();
        block();
        let end_time = Utc::now();
        let duration = end_time.signed_duration_since(start_time);
        self.send(&Metric::Timing { stat, val: duration.into() }, tags)
    }

    /// Send your own timing metric in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    ///   # extern crate chrono;
    ///   # extern crate dogstatsd;
    ///   # fn main() {
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.timing("timing", 350, &["tag:timing"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    ///   client.timing("timing", chrono::Duration::seconds(2), &["tag:timing"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    ///   # }
    /// ```
    pub fn timing<I, T, V>(&self, stat: &str, duration: V, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
              V: Into<DurationMeasurement>,
    {
        self.send(&Metric::Timing { stat, val: duration.into() }, tags)
    }

    /// Report an arbitrary value as a gauge
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.gauge("gauge", 12345, &["tag:gauge"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    ///   client.gauge("gauge", 123.45, &["tag:gauge"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn gauge<I, T, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
              V: Into<Measurement>,
    {
        self.send(&Metric::Gauge { stat, val: val.into() }, tags)
    }

    /// Report a value in a histogram
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.histogram("histogram", 67.890, &["tag:histogram"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn histogram<I, T, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
              V: Into<Measurement>,
    {
        self.send(&Metric::Histogram { stat, val: val.into() }, tags)
    }

    /// Report a value in a distribution
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.distribution("distribution", 67890, &["tag:distribution"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    ///   client.distribution("distribution", 67.890, &["tag:distribution"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn distribution<I, T, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
              V: Into<Measurement>,
    {
        self.send(&Metric::Distribution { stat, val: val.into() }, tags)
    }

    /// Report a value in a set
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.set("set", 13579, &["tag:set"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn set<I, T, V>(&self, stat: &str, val: V, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
              V: Into<Measurement>,
    {
        self.send(&Metric::Set { stat, val: val.into() }, tags)
    }

    /// Report the status of a service
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options, ServiceStatus, ServiceCheckOptions};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.service_check("redis.can_connect", ServiceStatus::OK, &["tag:service"], None)
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    ///
    ///   let options = ServiceCheckOptions {
    ///     hostname: Some("my-host.localhost"),
    ///     ..Default::default()
    ///   };
    ///   client.service_check("redis.can_connect", ServiceStatus::OK, &["tag:service"], Some(options))
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    ///
    ///   let all_options = ServiceCheckOptions {
    ///     hostname: Some("my-host.localhost"),
    ///     timestamp: Some(1510326433),
    ///     message: Some("Message about the check or service")
    ///   };
    ///   client.service_check("redis.can_connect", ServiceStatus::OK, &["tag:service"], Some(all_options))
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn service_check<I, T>(&self, stat: &str, val: ServiceStatus, tags: I, options: Option<ServiceCheckOptions>) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
    {
        let opt = options.unwrap_or(ServiceCheckOptions::default());
        self.send(&Metric::ServiceCheck { stat, val, opt }, tags)
    }

    /// Send a custom event as a title and a body
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.event("Event Title", "Event Body", &["tag:event"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn event<I, T>(&self, title: &str, text: &str, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
    {
        self.send(&Metric::Event { title, text }, tags)
    }

    fn send<I, T>(&self, metric: &Metric, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              T: AsRef<str>,
    {
        let formatted_metric = format_for_send(metric, &self.namespace, tags);
        self.socket.send_to(formatted_metric.as_slice(), &self.to_addr)?;
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
            from_addr: "127.0.0.1:0".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        };

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_new() {
        let client = Client::new(Options::default()).unwrap();
        let expected_client = Client {
            socket: UdpSocket::bind("127.0.0.1:0").unwrap(),
            from_addr: "127.0.0.1:0".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        };

        assert_eq!(expected_client, client)
    }

    #[test]
    fn test_send() {
        let options = Options::new("127.0.0.1:9001", "127.0.0.1:9002", "");
        let client = Client::new(options).unwrap();
        // Shouldn't panic or error
        client.send(&Metric::Gauge { stat: "gauge".into(), val: 1234.into() }, &["tag1", "tag2"]).unwrap();
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
