//! A Rust client for interacting with Dogstatsd

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
        let formatted_metric = format_for_send(metric.metric_type_format(), &self.namespace, tags);
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
