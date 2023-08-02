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
//! use dogstatsd::{Client, Options, OptionsBuilder};
//! use std::time::Duration;
//!
//! // Binds to a udp socket on an available ephemeral port on 127.0.0.1 for
//! // transmitting, and sends to  127.0.0.1:8125, the default dogstatsd
//! // address.
//! let default_options = Options::default();
//! let default_client = Client::new(default_options).unwrap();
//!
//! // Binds to 127.0.0.1:9000 for transmitting and sends to 10.1.2.3:8125, with a
//! // namespace of "analytics".
//! let custom_options = Options::new("127.0.0.1:9000", "10.1.2.3:8125", "analytics", vec![(String::new(), String::new())], None, None);
//! let custom_client = Client::new(custom_options).unwrap();
//!
//! // You can also use the OptionsBuilder API to avoid needing to specify every option.
//! let built_options = OptionsBuilder::new().from_addr(String::from("127.0.0.1:9001")).build();
//! let built_client = Client::new(built_options).unwrap();
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
#![deny(
warnings,
missing_debug_implementations,
missing_copy_implementations,
missing_docs
)]
extern crate chrono;

use std::borrow::Cow;
use std::future::Future;
use std::net::UdpSocket;
use std::os::unix::net::UnixDatagram;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::Utc;

pub use self::error::DogstatsdError;
use self::metrics::*;
pub use self::metrics::{ServiceCheckOptions, ServiceStatus};

mod error;
mod metrics;

/// A type alias for returning a unit type or an error
pub type DogstatsdResult = Result<(), DogstatsdError>;

const DEFAULT_FROM_ADDR: &str = "0.0.0.0:0";
const DEFAULT_TO_ADDR: &str = "127.0.0.1:8125";

/// The struct that represents the options available for the Dogstatsd client.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BatchingOptions {
    /// The maximum buffer size in bytes of a batch of events.
    pub max_buffer_size: usize,
    /// The maximum time before sending a batch of events.
    pub max_time: Duration,
    /// The maximum retry attempts if we fail to flush our buffer
    pub max_retry_attempts: usize,
    /// Upon retry, there is an exponential backoff, this value sets the starting value
    pub initial_retry_delay: u64,
}

/// The struct that represents the options available for the Dogstatsd client.
#[derive(Debug, PartialEq)]
pub struct Options {
    /// The address of the udp socket we'll bind to for sending.
    pub from_addr: String,
    /// The address of the udp socket we'll send metrics and events to.
    pub to_addr: String,
    /// A namespace to prefix all metrics with, joined with a '.'.
    pub namespace: String,
    /// Default tags to include with every request.
    pub default_tags: Vec<(String, String)>,
    /// OPTIONAL, if defined, will use UDS instead of UDP and will ignore UDP options
    pub socket_path: Option<String>,
    /// OPTIONAL, if defined, will utilize batching for sending metrics
    pub batching_options: Option<BatchingOptions>,
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
    ///           from_addr: "0.0.0.0:0".into(),
    ///           to_addr: "127.0.0.1:8125".into(),
    ///           namespace: String::new(),
    ///           default_tags: vec!(),
    ///           socket_path: None,
    ///           batching_options: None,
    ///       },
    ///       options
    ///   )
    /// ```
    fn default() -> Self {
        Options {
            from_addr: DEFAULT_FROM_ADDR.into(),
            to_addr: DEFAULT_TO_ADDR.into(),
            namespace: String::new(),
            default_tags: vec![],
            socket_path: None,
            batching_options: None,
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
    ///   let options = Options::new("127.0.0.1:9000", "127.0.0.1:9001", "", vec![(String::new(), String::new())], None, None);
    /// ```
    pub fn new(
        from_addr: &str,
        to_addr: &str,
        namespace: &str,
        default_tags: Vec<(String, String)>,
        socket_path: Option<String>,
        batching_options: Option<BatchingOptions>,
    ) -> Self {
        Options {
            from_addr: from_addr.into(),
            to_addr: to_addr.into(),
            namespace: namespace.into(),
            default_tags,
            socket_path,
            batching_options,
        }
    }
}

/// Struct that allows build an `Options` for available for the Dogstatsd client.
#[derive(Default, Debug)]
pub struct OptionsBuilder {
    /// The address of the udp socket we'll bind to for sending.
    from_addr: Option<String>,
    /// The address of the udp socket we'll send metrics and events to.
    to_addr: Option<String>,
    /// A namespace to prefix all metrics with, joined with a '.'.
    namespace: Option<String>,
    /// Default tags to include with every request.
    default_tags: Vec<(String, String)>,
    /// OPTIONAL, if defined, will use UDS instead of UDP and will ignore UDP options
    socket_path: Option<String>,
    /// OPTIONAL, if defined, will utilize batching for sending metrics
    batching_options: Option<BatchingOptions>,
}

impl OptionsBuilder {
    /// Create a new `OptionsBuilder` struct.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///
    ///   let options_builder = OptionsBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Will allow the builder to generate an `Options` struct with the provided value.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///
    ///   let options_builder = OptionsBuilder::new().from_addr(String::from("127.0.0.1:9000"));
    /// ```
    pub fn from_addr(&mut self, from_addr: String) -> &mut OptionsBuilder {
        self.from_addr = Some(from_addr);
        self
    }

    /// Will allow the builder to generate an `Options` struct with the provided value.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///
    ///   let options_builder = OptionsBuilder::new().to_addr(String::from("127.0.0.1:9001"));
    /// ```
    pub fn to_addr(&mut self, to_addr: String) -> &mut OptionsBuilder {
        self.to_addr = Some(to_addr);
        self
    }

    /// Will allow the builder to generate an `Options` struct with the provided value.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///
    ///   let options_builder = OptionsBuilder::new().namespace(String::from("mynamespace"));
    /// ```
    pub fn namespace(&mut self, namespace: String) -> &mut OptionsBuilder {
        self.namespace = Some(namespace);
        self
    }

    /// Will allow the builder to generate an `Options` struct with the provided value. Can be called multiple times to add multiple `default_tags` to the `Options`.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///
    ///   let options_builder = OptionsBuilder::new().default_tag(String::from("tag1"), String::from("tav1val")).default_tag(String::from("tag2"), String::from("tag2val"));
    /// ```
    pub fn default_tag(&mut self, default_tag_key: String, default_tag_value: String) -> &mut OptionsBuilder {
        self.default_tags.push((default_tag_key, default_tag_value));
        self
    }

    /// Will allow the builder to generate an `Options` struct with the provided value.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///
    ///   let options_builder = OptionsBuilder::new().default_tag(String::from("tag1:tav1val")).default_tag(String::from("tag2:tag2val"));
    /// ```
    pub fn socket_path(&mut self, socket_path: Option<String>) -> &mut OptionsBuilder {
        self.socket_path = socket_path;
        self
    }

    /// Will allow the builder to generate an `Options` struct with the provided value.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{ OptionsBuilder, BatchingOptions };
    ///   use std::time::Duration;
    ///
    ///   let options_builder = OptionsBuilder::new().batching_options(BatchingOptions { max_buffer_size: 8000, max_time: Duration::from_millis(3000), max_retry_attempts: 3, initial_retry_delay: 10 });
    /// ```
    pub fn batching_options(&mut self, batching_options: BatchingOptions) -> &mut OptionsBuilder {
        self.batching_options = Some(batching_options);
        self
    }

    /// Will construct an `Options` with all of the provided values and fall back to the default values if they aren't provided.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::OptionsBuilder;
    ///   use dogstatsd::Options;
    ///
    ///   let options = OptionsBuilder::new().namespace(String::from("mynamespace")).default_tag(String::from("tag1"), String::from("tav1val")).build();
    ///
    ///   assert_eq!(
    ///       Options {
    ///           from_addr: "0.0.0.0:0".into(),
    ///           to_addr: "127.0.0.1:8125".into(),
    ///           namespace: String::from("mynamespace"),
    ///           default_tags: vec![(String::from("tag1"), String::from("tav1val"))],
    ///           socket_path: None,
    ///           batching_options: None,
    ///       },
    ///       options
    ///   )
    /// ```
    pub fn build(&self) -> Options {
        Options::new(
            self.from_addr
                .as_ref()
                .unwrap_or(&String::from(DEFAULT_FROM_ADDR)),
            self.to_addr
                .as_ref()
                .unwrap_or(&String::from(DEFAULT_TO_ADDR)),
            self.namespace.as_ref().unwrap_or(&String::default()),
            self.default_tags.to_vec(),
            self.socket_path.clone(),
            self.batching_options,
        )
    }
}

#[derive(Debug)]
enum SocketType {
    Udp(UdpSocket),
    Uds(UnixDatagram),
    BatchableUdp(Mutex<Sender<batch_processor::Message>>),
    BatchableUds(Mutex<Sender<batch_processor::Message>>),
}

/// The client struct that handles sending metrics to the Dogstatsd server.
#[derive(Debug)]
pub struct Client {
    socket: SocketType,
    from_addr: String,
    to_addr: String,
    namespace: String,
    default_tags: Vec<u8>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        // Ignore `socket`, which will never be the same
        self.from_addr == other.from_addr
            && self.to_addr == other.to_addr
            && self.namespace == other.namespace
            && self.default_tags == other.default_tags
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        match &self.socket {
            SocketType::BatchableUdp(tx_channel) | SocketType::BatchableUds(tx_channel) => {
                // Destructing Client... If fails, ignore and keep going...
                let _ = tx_channel
                    .lock()
                    .unwrap()
                    .send(batch_processor::Message::Shutdown);
            }
            _ => {}
        }
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
        let fn_create_tx_channel = |socket: SocketType,
                                    batching_options: BatchingOptions,
                                    to_addr: String,
                                    socket_path: Option<String>|
         -> Mutex<Sender<batch_processor::Message>> {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || {
                batch_processor::process_events(batching_options, to_addr, socket, socket_path, rx);
            });
            Mutex::from(tx)
        };

        let socket = match options.socket_path {
            Some(socket_path) => {
                let uds_socket = UnixDatagram::unbound()?;
                uds_socket.set_nonblocking(true)?;
                uds_socket.connect(socket_path.clone())?;

                let wrapped_socket = SocketType::Uds(uds_socket);
                if let Some(batching_options) = options.batching_options {
                    SocketType::BatchableUds(fn_create_tx_channel(
                        wrapped_socket,
                        batching_options,
                        options.to_addr.clone(),
                        Some(socket_path),
                    ))
                } else {
                    wrapped_socket
                }
            }
            None => {
                let wrapped_socket = SocketType::Udp(UdpSocket::bind(&options.from_addr)?);
                if let Some(batching_options) = options.batching_options {
                    SocketType::BatchableUdp(fn_create_tx_channel(
                        wrapped_socket,
                        batching_options,
                        options.to_addr.clone(),
                        None,
                    ))
                } else {
                    wrapped_socket
                }
            }
        };

        let formatted_tags: Vec<String> = options.default_tags.iter().map(|(key, value)| format!("{}:{}", key, value)).collect();

        // SIMO

        Ok(Client {
            socket,
            from_addr: options.from_addr,
            to_addr: options.to_addr,
            namespace: options.namespace,
            default_tags: formatted_tags.join(",").into_bytes(),
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(&CountMetric::Incr(stat.into().as_ref(), 1), tags)
    }

    /// Increment a StatsD counter by the provided amount
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.incr_by_value("counter", 123, &["tag:counter"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn incr_by_value<'a, I, S, T>(&self, stat: S, value: i64, tags: I) -> DogstatsdResult
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: AsRef<str>,
    {
        self.send(&CountMetric::Incr(stat.into().as_ref(), value), tags)
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(&CountMetric::Decr(stat.into().as_ref(), 1), tags)
    }

    /// Decrement a StatsD counter by the provided amount
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///
    ///   let client = Client::new(Options::default()).unwrap();
    ///   client.decr_by_value("counter", 23, &["tag:counter"])
    ///       .unwrap_or_else(|e| println!("Encountered error: {}", e));
    /// ```
    pub fn decr_by_value<'a, I, S, T>(&self, stat: S, value: i64, tags: I) -> DogstatsdResult
    where
        I: IntoIterator<Item = T>,
        S: Into<Cow<'a, str>>,
        T: AsRef<str>,
    {
        self.send(&CountMetric::Decr(stat.into().as_ref(), value), tags)
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
        where
            I: IntoIterator<Item=T>,
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
    ///   }).unwrap_or_else(|(_, e)| println!("Encountered error: {}", e))
    /// ```
    pub fn time<'a, F, O, I, S, T>(
        &self,
        stat: S,
        tags: I,
        block: F,
    ) -> Result<O, (O, DogstatsdError)>
        where
            F: FnOnce() -> O,
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        let start_time = Utc::now();
        let output = block();
        let end_time = Utc::now();
        let stat = stat.into();
        let metric = TimeMetric::new(stat.as_ref(), &start_time, &end_time);
        match self.send(&metric, tags) {
            Ok(()) => Ok(output),
            Err(error) => Err((output, error)),
        }
    }

    /// Time how long it takes for an async block of code to execute.
    ///
    /// # Examples
    ///
    /// ```
    ///   use dogstatsd::{Client, Options};
    ///   use std::thread;
    ///   use std::time::Duration;
    ///
    /// # async fn do_work() {}
    ///   async fn timer() {
    ///       let client = Client::new(Options::default()).unwrap();
    ///       client.async_time("timer", &["tag:time"], do_work)
    ///       .await
    ///       .unwrap_or_else(|(_, e)| println!("Encountered error: {}", e))
    ///   }
    /// ```
    pub async fn async_time<'a, Fn, Fut, O, I, S, T>(
        &self,
        stat: S,
        tags: I,
        block: Fn,
    ) -> Result<O, (O, DogstatsdError)>
        where
            Fn: FnOnce() -> Fut,
            Fut: Future<Output=O>,
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        let start_time = Utc::now();
        let output = block().await;
        let end_time = Utc::now();
        let stat = stat.into();
        match self.send(
            &TimeMetric::new(stat.as_ref(), &start_time, &end_time),
            tags,
        ) {
            Ok(()) => Ok(output),
            Err(error) => Err((output, error)),
        }
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
        where
            I: IntoIterator<Item=T>,
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            SS: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(
            &GaugeMetric::new(stat.into().as_ref(), val.into().as_ref()),
            tags,
        )
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            SS: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(
            &HistogramMetric::new(stat.into().as_ref(), val.into().as_ref()),
            tags,
        )
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            SS: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(
            &DistributionMetric::new(stat.into().as_ref(), val.into().as_ref()),
            tags,
        )
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            SS: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(
            &SetMetric::new(stat.into().as_ref(), val.into().as_ref()),
            tags,
        )
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
    pub fn service_check<'a, I, S, T>(
        &self,
        stat: S,
        val: ServiceStatus,
        tags: I,
        options: Option<ServiceCheckOptions>,
    ) -> DogstatsdResult
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        let unwrapped_options = options.unwrap_or_default();
        self.send(
            &ServiceCheck::new(stat.into().as_ref(), val, unwrapped_options),
            tags,
        )
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
        where
            I: IntoIterator<Item=T>,
            S: Into<Cow<'a, str>>,
            SS: Into<Cow<'a, str>>,
            T: AsRef<str>,
    {
        self.send(
            &Event::new(title.into().as_ref(), text.into().as_ref()),
            tags,
        )
    }

    fn send<I, M, S>(&self, metric: &M, tags: I) -> DogstatsdResult
        where
            I: IntoIterator<Item=S>,
            M: Metric,
            S: AsRef<str>,
    {
        let formatted_metric = format_for_send(metric, &self.namespace, tags, &self.default_tags);
        match &self.socket {
            SocketType::Udp(socket) => {
                socket.send_to(formatted_metric.as_slice(), &self.to_addr)?;
            }
            SocketType::Uds(socket) => {
                socket.send(formatted_metric.as_slice())?;
            }
            SocketType::BatchableUdp(tx_channel) | SocketType::BatchableUds(tx_channel) => {
                tx_channel
                    .lock()
                    .expect("Mutex poisoned...")
                    .send(batch_processor::Message::Data(formatted_metric))
                    .unwrap_or_else(|error| {
                        println!("Exception occurred when writing to channel: {:?}", error);
                    });
            }
        }
        Ok(())
    }
}

mod batch_processor {
    use crate::{BatchingOptions, SocketType};
    use retry::{delay::jitter, delay::Exponential, retry};
    use std::sync::mpsc::Receiver;
    use std::time::SystemTime;

    pub(crate) enum Message {
        Data(Vec<u8>),
        Shutdown,
    }

    fn send_to_socket_with_retries(
        batching_options: &BatchingOptions,
        socket: &SocketType,
        data: &Vec<u8>,
        to_addr: &String,
        socket_path: &Option<String>,
    ) {
        retry(
            Exponential::from_millis(batching_options.initial_retry_delay)
                .map(jitter)
                .take(batching_options.max_retry_attempts),
            || {
                match socket {
                    SocketType::Udp(socket) => {
                        if let Err(error) = socket.send_to(data.as_slice(), to_addr) {
                            println!(
                                "Exception occurred when writing to UDP socket: {:?} {}",
                                error,
                                data.len()
                            );
                            return Err(error);
                        }
                    }
                    SocketType::Uds(socket) => {
                        if let Err(error) = socket.send(data.as_slice()) {
                            println!(
                                "Exception occurred when writing to UDS socket: {:?} {}",
                                error,
                                data.len()
                            );

                            // Per https://doc.rust-lang.org/stable/std/os/unix/net/struct.UnixDatagram.html#method.send
                            // If send fails, it is due to a connection issue, so just attempt
                            // to reconnect
                            let socket_path_unwrapped = socket_path
                                .as_ref()
                                .expect("Only invoked if socket path is defined.");
                            println!(
                                "Attempting to reconnect to socket... {}",
                                socket_path_unwrapped
                            );
                            socket.connect(socket_path_unwrapped)?;

                            return Err(error);
                        }
                    }
                    SocketType::BatchableUdp(_tx_channel)
                    | SocketType::BatchableUds(_tx_channel) => {
                        panic!("Logic Error - socket type should not be batchable.");
                    }
                }

                Ok(())
            },
        )
        .unwrap_or_else(|error| {
            println!(
                "Failed to send within retry policy... Dropping metrics: {:?}",
                error
            )
        });
    }

    pub(crate) fn process_events(
        batching_options: BatchingOptions,
        to_addr: String,
        socket: SocketType,
        socket_path: Option<String>,
        rx: Receiver<Message>,
    ) {
        let mut last_updated = SystemTime::now();
        let mut buffer: Vec<u8> = vec![];

        loop {
            match rx.recv() {
                Ok(Message::Data(data)) => {
                    for ch in data {
                        buffer.push(ch);
                    }
                    buffer.push(b'\n');

                    let current_time = SystemTime::now();
                    if buffer.len() >= batching_options.max_buffer_size
                        || last_updated + batching_options.max_time < current_time
                    {
                        send_to_socket_with_retries(
                            &batching_options,
                            &socket,
                            &buffer,
                            &to_addr,
                            &socket_path,
                        );
                        buffer.clear();
                        last_updated = current_time;
                    }
                }
                Ok(Message::Shutdown) => {
                    send_to_socket_with_retries(
                        &batching_options,
                        &socket,
                        &buffer,
                        &to_addr,
                        &socket_path,
                    );
                    buffer.clear();
                }
                Err(e) => {
                    println!("Exception occurred when reading from channel: {:?}", e);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use metrics::GaugeMetric;

    use super::*;

    #[test]
    fn test_options_default() {
        let options = Options::default();
        let expected_options = Options {
            from_addr: DEFAULT_FROM_ADDR.into(),
            to_addr: DEFAULT_TO_ADDR.into(),
            namespace: String::new(),
            ..Default::default()
        };

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_options_builder_none() {
        let options = OptionsBuilder::new().build();
        let expected_options = Options {
            from_addr: DEFAULT_FROM_ADDR.into(),
            to_addr: DEFAULT_TO_ADDR.into(),
            namespace: String::new(),
            ..Default::default()
        };

        assert_eq!(expected_options, options);
    }

    #[test]
    fn teset_options_builder_all() {
        let options = OptionsBuilder::new()
            .from_addr("127.0.0.2:0".into())
            .to_addr("127.0.0.2:8125".into())
            .namespace("mynamespace".into())
            .default_tag(String::from("tag1"), String::from("tag1val"))
            .build();
        let expected_options = Options {
            from_addr: "127.0.0.2:0".into(),
            to_addr: "127.0.0.2:8125".into(),
            namespace: "mynamespace".into(),
            default_tags: vec![("tag1".into(), "tag1val".into())].to_vec(),
            socket_path: None,
            batching_options: None,
        };

        assert_eq!(expected_options, options);
    }

    #[test]
    fn test_new() {
        let client = Client::new(Options::default()).unwrap();
        let expected_client = Client {
            socket: SocketType::Udp(UdpSocket::bind(DEFAULT_FROM_ADDR).unwrap()),
            from_addr: DEFAULT_FROM_ADDR.into(),
            to_addr: DEFAULT_TO_ADDR.into(),
            namespace: String::new(),
            default_tags: String::new().into_bytes(),
        };

        assert_eq!(expected_client, client)
    }

    #[test]
    fn test_new_default_tags() {
        let options = Options::new(
            DEFAULT_FROM_ADDR,
            DEFAULT_TO_ADDR,
            "",
            vec![(String::from("tag1"), String::from("tag1val"))],
            None,
            None,
        );
        let client = Client::new(options).unwrap();
        let expected_client = Client {
            socket: SocketType::Udp(UdpSocket::bind(DEFAULT_FROM_ADDR).unwrap()),
            from_addr: DEFAULT_FROM_ADDR.into(),
            to_addr: DEFAULT_TO_ADDR.into(),
            namespace: String::new(),
            default_tags: String::from("tag1:tag1val").into_bytes(),
        };

        assert_eq!(expected_client, client)
    }

    #[test]
    fn test_send() {
        let options = Options::new("127.0.0.1:9001", "127.0.0.1:9002", "", vec![], None, None);
        let client = Client::new(options).unwrap();
        // Shouldn't panic or error
        client
            .send(
                &GaugeMetric::new("gauge".into(), "1234".into()),
                &["tag1", "tag2"],
            )
            .unwrap();
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;

    use super::*;

    use self::test::Bencher;

    use self::test::Bencher;

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
            client
                .histogram("bench.histogram", &i.to_string(), &tags)
                .unwrap();
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
            client
                .distribution("bench.distribution", &i.to_string(), &tags)
                .unwrap();
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
            message: Some("Message about the check or service"),
        };
        b.iter(|| {
            client
                .service_check(
                    "bench.service_check",
                    ServiceStatus::Critical,
                    &tags,
                    Some(all_options),
                )
                .unwrap();
        })
    }

    #[bench]
    fn bench_event(b: &mut Bencher) {
        let options = Options::default();
        let client = Client::new(options).unwrap();
        let tags = vec!["name1:value1"];
        b.iter(|| {
            client
                .event("Test Event Title", "Test Event Message", &tags)
                .unwrap();
        })
    }
}
