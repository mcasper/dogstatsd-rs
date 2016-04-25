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
    ///       },
    ///       options
    ///   )
    /// ```
    pub fn default() -> Self {
        Options {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
        }
    }
}

/// The client struct that handles sending metrics to the Dogstatsd server.
#[derive(Debug, PartialEq)]
pub struct Client {
    from_addr: String,
    to_addr: String,
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
    ///   client.incr("counter").unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn incr(&self, stat: &str) -> DogstatsdResult {
        self.send(CountMetric::Incr(stat.into()))
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
    ///   client.decr("counter").unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn decr(&self, stat: &str) -> DogstatsdResult {
        self.send(CountMetric::Decr(stat.into()))
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
    ///   client.time("timer", {||
    ///       thread::sleep(Duration::from_millis(200))
    ///   }).unwrap_or_else(|e| println!("Encountered error: {}", e))
    /// ```
    pub fn time<F: FnOnce()>(&self, stat: &str, block: F) -> DogstatsdResult {
        let start_time = UTC::now();
        block();
        let end_time = UTC::now();

        self.send(TimeMetric::new(stat, start_time, end_time))
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
    ///   client.timing("timing", 350).unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn timing(&self, stat: &str, ms: i64) -> DogstatsdResult {
        self.send(TimingMetric::new(stat, ms))
    }

    /// Report an arbitrary value as a gauge
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.gauge("gauge", "12345").unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn gauge(&self, stat: &str, val: &str) -> DogstatsdResult {
        self.send(GaugeMetric::new(stat, val))
    }

    /// Report a value in a histogram
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.histogram("histogram", "67890").unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn histogram(&self, stat: &str, val: &str) -> DogstatsdResult {
        self.send(HistogramMetric::new(stat, val))
    }

    /// Report a value in a set
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.set("set", "13579").unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn set(&self, stat: &str, val: &str) -> DogstatsdResult {
        self.send(SetMetric::new(stat, val))
    }

    /// Send a custom event as a title and a body
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default());
    ///   client.event("Event Title", "Event Body").unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn event(&self, title: &str, text: &str) -> DogstatsdResult {
        self.send(Event::new(title, text))
    }

    fn send<M: Metric>(&self, metric: M) -> DogstatsdResult {
        let socket = try!(self.socket());
        try!(socket.send_to(metric.format_for_send().as_bytes(), &self.to_addr[..]));
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
        };

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_new() {
        let client = Client::new(Options::default());
        let expected_client = Client {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
        };

        assert_eq!(expected_client, client)
    }

    #[test]
    fn test_socket() {
        let client = Client::new(Options::default());
        // Shouldn't panic or error
        client.socket().unwrap();
    }
}
