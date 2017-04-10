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
//! // Binds to a udp socket on 127.0.0.1:8126 for transmitting, and sends to
//! // 127.0.0.1:8125, the default dogstatsd address.
//! let default_options = Options::default();
//! let default_client = Client::new(default_options);
//!
//! // Binds to 127.0.0.1:9000 for transmitting and sends to 10.1.2.3:8125, with a
//! // namespace of "analytics".
//! let custom_options = Options::new("127.0.0.1:9000", "10.1.2.3:8125", "analytics");
//! let custom_client = Client::new(custom_options);
//! ```
//!
//! Start sending metrics:
//!
//! ```
//! use dogstatsd::{Client, Options};
//!
//! let client = Client::new(Options::default());
//!
//! // Increment a counter
//! client.incr("my_counter", vec![]).unwrap();
//!
//! // Decrement a counter
//! client.decr("my_counter", vec![]).unwrap();
//!
//! // Time a block of code (reports in ms)
//! client.time("my_time", vec![], || {
//!     // Some time consuming code
//! }).unwrap();
//!
//! // Report your own timing in ms
//! client.timing("my_timing", 500, vec![]).unwrap();
//!
//! // Report an arbitrary value (a gauge)
//! client.gauge("my_gauge", "12345", vec![]).unwrap();
//!
//! // Report a sample of a histogram
//! client.histogram("my_histogram", "67890", vec![]).unwrap();
//!
//! // Report a member of a set
//! client.set("my_set", "13579", vec![]).unwrap();
//!
//! // Send a custom event
//! client.event("My Custom Event Title", "My Custom Event Body", vec![]).unwrap();
//!
//! // Add tags to any metric by passing a Vec<String> of tags to apply
//! client.gauge("my_gauge", "12345", vec!["tag:1".into(), "tag:2".into()]).unwrap();
//! ```

#![cfg_attr(feature = "unstable", feature(test))]
#![deny(warnings, missing_debug_implementations, missing_copy_implementations, missing_docs)]
extern crate chrono;

use chrono::UTC;
use std::net::UdpSocket;

mod error;
use self::error::DogstatsdError;

mod metrics;
use self::metrics::*;

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

impl Options {
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
    ///           from_addr: "127.0.0.1:8126".into(),
    ///           to_addr: "127.0.0.1:8125".into(),
    ///           namespace: String::new(),
    ///       },
    ///       options
    ///   )
    /// ```
    pub fn default() -> Self {
        Options {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        }
    }

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
#[derive(Debug, PartialEq)]
pub struct Client {
    from_addr: String,
    to_addr: String,
    namespace: String,
}

impl Client {
    /// Create a new client from an options struct.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    /// ```
    pub fn new(options: Options) -> Self {
        Client {
            from_addr: options.from_addr,
            to_addr: options.to_addr,
            namespace: options.namespace,
        }
    }

    /// Increment a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///
    ///   let client = Client::new(Options::default());
    ///   client.incr("counter", vec!["tag:counter".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn incr<S: Into<String>>(&self, stat: S, tags: Vec<String>) -> DogstatsdResult {
        self.send(CountMetric::Incr(stat.into()), tags)
    }

    /// Decrement a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///
    ///   let client = Client::new(Options::default());
    ///   client.decr("counter", vec!["tag:counter".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn decr<S: Into<String>>(&self, stat: S, tags: Vec<String>) -> DogstatsdResult {
        self.send(CountMetric::Decr(stat.into()), tags)
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
    ///
    ///   let client = Client::new(Options::default());
    ///   client.time("timer", vec!["tag:time".into()], || {
    ///       thread::sleep(Duration::from_millis(200))
    ///   }).unwrap_or_else(|e| println!("Encountered error: {}", e))
    /// ```
    pub fn time<S: Into<String>, F: FnOnce()>(&self, stat: S, tags: Vec<String>, block: F) -> DogstatsdResult {
        let start_time = UTC::now();
        block();
        let end_time = UTC::now();

        self.send(TimeMetric::new(stat.into(), start_time, end_time), tags)
    }

    /// Send your own timing metric in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///
    ///   let client = Client::new(Options::default());
    ///   client.timing("timing", 350, vec!["tag:timing".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn timing<S: Into<String>>(&self, stat: S, ms: i64, tags: Vec<String>) -> DogstatsdResult {
        self.send(TimingMetric::new(stat.into(), ms), tags)
    }

    /// Report an arbitrary value as a gauge
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.gauge("gauge", "12345", vec!["tag:gauge".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn gauge<S: Into<String>>(&self, stat: S, val: S, tags: Vec<String>) -> DogstatsdResult {
        self.send(GaugeMetric::new(stat.into(), val.into()), tags)
    }

    /// Report a value in a histogram
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.histogram("histogram", "67890", vec!["tag:histogram".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn histogram<S: Into<String>>(&self, stat: S, val: S, tags: Vec<String>) -> DogstatsdResult {
        self.send(HistogramMetric::new(stat.into(), val.into()), tags)
    }

    /// Report a value in a set
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.set("set", "13579", vec!["tag:set".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn set<S: Into<String>>(&self, stat: S, val: S, tags: Vec<String>) -> DogstatsdResult {
        self.send(SetMetric::new(stat.into(), val.into()), tags)
    }

    /// Send a custom event as a title and a body
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.event("Event Title", "Event Body", vec!["tag:event".into()])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn event<S: Into<String>>(&self, title: S, text: S, tags: Vec<String>) -> DogstatsdResult {
        self.send(Event::new(title.into(), text.into()), tags)
    }

    fn send<M: Metric>(&self, metric: M, tags: Vec<String>) -> DogstatsdResult {
        let socket = try!(self.socket());
        let formatted_metric = format_for_send(metric.metric_type_format(), &self.namespace[..], tags.as_slice());
        try!(socket.send_to(formatted_metric.as_bytes(), &self.to_addr[..]));
        Ok(())
    }

    fn socket(&self) -> Result<UdpSocket, DogstatsdError> {
        let socket = try!(UdpSocket::bind(&self.from_addr[..]));
        Ok(socket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let options = Options::default();
        let expected_options = Options {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        };

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_new() {
        let client = Client::new(Options::default());
        let expected_client = Client {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
            namespace: String::new(),
        };

        assert_eq!(expected_client, client)
    }

    #[test]
    fn test_socket() {
        let client = Client::new(Options::default());
        // Shouldn't panic or error
        client.socket().unwrap();
    }

    use metrics::GaugeMetric;
    #[test]
    fn test_send() {
        let options = Options::new("127.0.0.1:9001", "127.0.0.1:9002", "");
        let client = Client::new(options);
        // Shouldn't panic or error
        client.send(GaugeMetric::new("gauge".into(), "1234".into()), vec!["tag1".into(), "tag2".into()]).unwrap();
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
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        b.iter(|| {
            client.incr("bench.incr", tags.clone()).unwrap();
        })
    }

    #[bench]
    fn bench_decr(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        b.iter(|| {
            client.decr("bench.decr", tags.clone()).unwrap();
        })
    }

    #[bench]
    fn bench_timing(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        let mut i = 0;
        b.iter(|| {
            client.timing("bench.timing", i, tags.clone()).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_gauge(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        let mut i = 0;
        b.iter(|| {
            client.gauge("bench.timing", &i.to_string(), tags.clone()).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_histogram(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        let mut i = 0;
        b.iter(|| {
            client.histogram("bench.timing", &i.to_string(), tags.clone()).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_set(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        let mut i = 0;
        b.iter(|| {
            client.set("bench.timing", &i.to_string(), tags.clone()).unwrap();
            i += 1;
        })
    }

    #[bench]
    fn bench_event(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options);
        let tags = vec!["name1:value1".to_string(), "name2:value2".to_string()];
        b.iter(|| {
            client.event("Test Event Title", "Test Event Message", tags.clone()).unwrap();
        })
    }
}
