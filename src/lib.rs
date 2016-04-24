extern crate chrono;

use chrono::UTC;
use std::net::UdpSocket;

mod error;
use self::error::DogstatsdError;

mod metrics;
use self::metrics::*;

#[derive(Debug, PartialEq)]
pub struct DogstatsdOptions {
    pub from_addr: String,
    pub to_addr: String,
}

impl DogstatsdOptions {
    pub fn default() -> Self {
        DogstatsdOptions {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Dogstatsd {
    from_addr: String,
    to_addr: String,
}

impl Dogstatsd {
    /// Spawn a new Dogstatsd handle
    pub fn new(options: DogstatsdOptions) -> Self {
        Dogstatsd {
            from_addr: options.from_addr,
            to_addr: options.to_addr,
        }
    }

    /// Increment a counter
    pub fn incr(&self, stat: &str) -> Result<(), DogstatsdError> {
        self.send(CountMetric::Incr(stat.into()))
    }

    /// Decrement a counter
    pub fn decr(&self, stat: &str) -> Result<(), DogstatsdError> {
        self.send(CountMetric::Decr(stat.into()))
    }

    /// Time a block of code
    pub fn time<F: FnOnce()>(&self, stat: &str, block: F) -> Result<(), DogstatsdError> {
        let start_time = UTC::now();
        block();
        let end_time = UTC::now();

        self.send(TimeMetric::new(stat, start_time, end_time))
    }

    /// Send a timing metric
    pub fn timing(&self, stat: &str, ms: i64) -> Result<(), DogstatsdError> {
        self.send(TimingMetric::new(stat, ms))
    }

    /// Report an arbitrary value (a gauge)
    pub fn gauge(&self, stat: &str, val: &str) -> Result<(), DogstatsdError> {
        self.send(GaugeMetric::new(stat, val))
    }

    /// Report a value in a histogram
    pub fn histogram(&self, stat: &str, val: &str) -> Result<(), DogstatsdError> {
        self.send(HistogramMetric::new(stat, val))
    }

    /// Report a value in a set
    pub fn set(&self, stat: &str, val: &str) -> Result<(), DogstatsdError> {
        self.send(SetMetric::new(stat, val))
    }

    /// Send a custom event
    pub fn event(&self, title: &str, text: &str) -> Result<(), DogstatsdError> {
        self.send(Event::new(title, text))
    }

    fn send<M: Metric>(&self, metric: M) -> Result<(), DogstatsdError> {
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
    use std::net::UdpSocket;

    #[test]
    fn test_options_default() {
        let options = DogstatsdOptions::default();
        let expected_options = DogstatsdOptions {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
        };

        assert_eq!(expected_options, options)
    }

    #[test]
    fn test_new() {
        let client = Dogstatsd::new(DogstatsdOptions::default());
        let expected_client = Dogstatsd {
            from_addr: "127.0.0.1:8126".into(),
            to_addr: "127.0.0.1:8125".into(),
        };

        assert_eq!(expected_client, client)
    }
}
