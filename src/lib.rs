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
//! let custom_options = Options::new("127.0.0.1:9000", "10.1.2.3:8125", "analytics").unwrap();
//! let custom_client = Client::new(custom_options).unwrap();
//! ```
//!
//! Start sending metrics:
//!
//! ```
//! use dogstatsd::{Client, Options, ServiceCheckOptions, ServiceStatus};
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
//! client.gauge("my_gauge", "12345", tags).unwrap();
//!
//! // Report a sample of a histogram
//! client.histogram("my_histogram", "67890", tags).unwrap();
//!
//! // Report a sample of a distribution
//! client.distribution("distribution", "67890", tags).unwrap();
//!
//! // Report a member of a set
//! client.set("my_set", "13579", tags).unwrap();
//!
//! // Report a service check
//! let service_check_options = ServiceCheckOptions {
//!   hostname: Some("my-host.localhost"),
//!   ..Default::default()
//! };
//! client.service_check("redis.can_connect", ServiceStatus::OK, tags, Some(service_check_options)).unwrap();
//!
//! // Send a custom event
//! client.event("My Custom Event Title", "My Custom Event Body", tags).unwrap();
//! ```

#![cfg_attr(feature = "unstable", feature(test))]
#![deny(warnings, missing_debug_implementations, missing_copy_implementations, missing_docs)]
extern crate chrono;

use chrono::Utc;
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs, Ipv4Addr};
use std::borrow::Cow;
use std::io;

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
    pub from_addr: SocketAddr,
    /// The address of the udp socket we'll send metrics and events to.
    pub to_addr: SocketAddr,
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
    ///           from_addr: ([127, 0, 0, 1], 0).into(),
    ///           to_addr: ([127, 0, 0, 1], 8125).into(),
    ///           namespace: String::new(),
    ///       },
    ///       options
    ///   )
    /// ```
    fn default() -> Self {
        let localhost = Ipv4Addr::new(127, 0, 0, 1);

        // ToSocketAddrs on (Ipv4Addr, u16) is infallible:
        Self::new((localhost, 0), (localhost, 8125), "").unwrap()
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
    ///
    pub fn new(from_addr: impl ToSocketAddrs, to_addr: impl ToSocketAddrs, namespace: &str) -> Result<Self, io::Error> {
        fn resolve(addr: impl ToSocketAddrs) -> Result<SocketAddr, io::Error> {
            addr.to_socket_addrs()?
                .nth(0)
                .ok_or(io::Error::new(io::ErrorKind::AddrNotAvailable, "cannot resolve address"))
        }

        Ok(Options {
            from_addr: resolve(from_addr)?,
            to_addr: resolve(to_addr)?,
            namespace: namespace.to_owned(),
        })
    }
}

/// The client struct that handles sending metrics to the Dogstatsd server.
#[derive(Debug)]
pub struct Client {
    socket: UdpSocket,
    from_addr: SocketAddr,
    to_addr: SocketAddr,
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
    pub fn incr<'a, I, S, T>(&self, stat: S, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&CountMetric::Incr(stat.into().as_ref()), tags)
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
    pub fn decr<'a, I, S, T>(&self, stat: S, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&CountMetric::Decr(stat.into().as_ref()), tags)
    }

    /// Make an arbitrary change to a StatsD counter
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.count("counter", 42, &["tag:counter"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn count<'a, I, S, T>(&self, stat: S, count: i64, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&CountMetric::Arbitrary(stat.into().as_ref(), count), tags)
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
    pub fn time<'a, F, I, S, T>(&self, stat: S, tags: I, block: F) -> DogstatsdResult
        where F: FnOnce(),
              I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        let start_time = Utc::now();
        block();
        let end_time = Utc::now();
        self.send(&TimeMetric::new(stat.into().as_ref(), &start_time, &end_time), tags)
    }

    /// Send your own timing metric in milliseconds
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.timing("timing", 350, &["tag:timing"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn timing<'a, I, S, T>(&self, stat: S, ms: i64, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&TimingMetric::new(stat.into().as_ref(), ms), tags)
    }

    /// Report an arbitrary value as a gauge
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.gauge("gauge", "12345", &["tag:gauge"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn gauge<'a, I, S, SS, T>(&self, stat: S, val: SS, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              SS: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&GaugeMetric::new(stat.into().as_ref(), val.into().as_ref()), tags)
    }

    /// Report a value in a histogram
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.histogram("histogram", "67890", &["tag:histogram"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn histogram<'a, I, S, SS, T>(&self, stat: S, val: SS, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              SS: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&HistogramMetric::new(stat.into().as_ref(), val.into().as_ref()), tags)
    }

    /// Report a value in a distribution
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.distribution("distribution", "67890", &["tag:distribution"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn distribution<'a, I, S, SS, T>(&self, stat: S, val: SS, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              SS: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&DistributionMetric::new(stat.into().as_ref(), val.into().as_ref()), tags)
    }

    /// Report a value in a set
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.set("set", "13579", &["tag:set"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn set<'a, I, S, SS, T>(&self, stat: S, val: SS, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              SS: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&SetMetric::new(stat.into().as_ref(), val.into().as_ref()), tags)
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
    pub fn service_check<'a, I, S, T>(&self, stat: S, val: ServiceStatus, tags: I, options: Option<ServiceCheckOptions>) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        let unwrapped_options = options.unwrap_or_default();
        self.send(&ServiceCheck::new(stat.into().as_ref(), val, unwrapped_options), tags)
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
    pub fn event<'a, I, S, SS, T>(&self, title: S, text: SS, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=T>,
              S: Into<Cow<'a, str>>,
              SS: Into<Cow<'a, str>>,
              T: AsRef<str>,
    {
        self.send(&Event::new(title.into().as_ref(), text.into().as_ref()), tags)
    }

    fn send<I, M, S>(&self, metric: &M, tags: I) -> DogstatsdResult
        where I: IntoIterator<Item=S>,
              M: Metric,
              S: AsRef<str>,
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
        let expected_options = Options::new("127.0.0.1:0", "127.0.0.1:8125", "").unwrap();

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_new() {
        let client = Client::new(Options::default()).unwrap();
        let expected_client = Client {
            socket: UdpSocket::bind("127.0.0.1:0").unwrap(),
            from_addr: ([127, 0, 0, 1], 0).into(),
            to_addr: ([127, 0, 0, 1], 8125).into(),
            namespace: String::new(),
        };

        assert_eq!(expected_client, client)
    }

    use metrics::GaugeMetric;
    #[test]
    fn test_send() {
        let options = Options::new("127.0.0.1:9001", "127.0.0.1:9002", "").unwrap();
        let client = Client::new(options).unwrap();
        // Shouldn't panic or error
        client.send(&GaugeMetric::new("gauge".into(), "1234".into()), &["tag1", "tag2"]).unwrap();
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
    fn bench_count(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = &["name1:value1"];
        let mut i = 0;
        b.iter(|| {
            client.count("bench.count", i, tags).unwrap();
            i += 1;
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
        let all_options = ServiceCheckOptions {
            hostname: Some("my-host.localhost"),
            timestamp: Some(1510326433),
            message: Some("Message about the check or service")
        };
        b.iter(|| {
            client.service_check("bench.service_check", ServiceStatus::Critical, &tags, Some(all_options)).unwrap();
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
