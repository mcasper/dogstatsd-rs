
use std::fmt;
use std::borrow::Cow;
use chrono;

pub enum Metric<'a> {
    // Tracks how many times something happened per second.
    Count { stat: Stat<'a>, val: i64 },
    /// Measures the value of a metric at a particular time.
    Gauge { stat: Stat<'a>, val: Measurement },
    /// Tracks the statistical distribution of a set of values on each host.
    Histogram { stat: Stat<'a>, val: Measurement },
    /// Tracks the statistical distribution of a set of values across infrastructure.
    Distribution { stat: Stat<'a>, val: Measurement },
    /// Counts the number of unique elements in a group.
    Set { stat: Stat<'a>, val: SetMeasurement<'a> },
    /// Sends timing information.
    Timing { stat: Stat<'a>, val: DurationMeasurement },
    /// Sends the provided ServiceCheck.
    ServiceCheck { stat: Stat<'a>, status: ServiceStatus, opt: ServiceCheckOptions<'a> },
    /// Sends an event with the provided title and text.
    Event { title: &'a str, text: &'a str, opt: EventOptions<'a> },
}

impl<'a> Metric<'a> {

    pub fn uses_namespace(&self) -> bool {
        use Metric::*;
        match self {
            Event { .. } => false,
            ServiceCheck { .. } => false,
            _ => true,
        }
    }

    pub fn metric_type_format(&self) -> String {
        use Metric::*;
        match self {
            Count { stat, val }         => format!("{}:{}|c", stat, val),
            Gauge { stat, val }         => format!("{}:{}|g", stat, val),
            Histogram { stat, val }     => format!("{}:{}|h", stat, val),
            Distribution { stat, val }  => format!("{}:{}|d", stat, val),
            Set { stat, val }           => format!("{}:{}|s", stat, val),
            Timing { stat, val }        => format!("{}:{}|ms", stat, val),
            
            Event { title, text, opt } => {

                let mut buf = format!("_e{{{},{}}}:{}|{}", title.len(), text.len(), title, text);

                if let Some(timestamp) = opt.timestamp {
                    buf.push_str(&format!("|d:{}", timestamp));
                }

                if let Some(hostname) = opt.hostname {
                    buf.push_str(&format!("|h:{}", hostname));
                }

                if let Some(aggregation_key) = opt.aggregation_key {
                    buf.push_str(&format!("|k:{}", aggregation_key));
                }

                if let Some(priority) = opt.priority {
                    buf.push_str(&format!("|p:{}", priority));
                }

                if let Some(source_type) = opt.source_type {
                    buf.push_str(&format!("|s:{}", source_type));
                }

                if let Some(alert_type) = opt.alert_type {
                    buf.push_str(&format!("|t:{}", alert_type));
                }

                buf
            }
            
            ServiceCheck { stat, status, opt } => {
                let mut buf = String::with_capacity(6 + stat.len() + opt.len());

                buf.push_str("_sc|");
                buf.push_str(stat);
                buf.push_str("|");
                buf.push_str(&format!("{}", *status as u8));

                if let Some(timestamp) = opt.timestamp {
                    buf.push_str(&format!("|d:{}", timestamp));
                }

                if let Some(hostname) = opt.hostname {
                    buf.push_str(&format!("|h:{}", hostname));
                }

                if let Some(message) = opt.message {
                    buf.push_str(&format!("|m:{}", message));
                }

                buf
            }
        }
    }
}

pub type Stat<'a> = &'a str;

/// Represents most values that can be reported by the client.
#[derive(Clone, Copy, Debug)]
pub enum Measurement {
    Int(i64),
    Float(f64),
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Measurement::*;
        match self {
            Int(val) => write!(f, "{}", val),
            Float(val) => write!(f, "{:.6}", val),
        }
    }
}

macro_rules! measurement_from {
    ($t:ty, Int) => (
        impl From<$t> for Measurement {
            fn from(value: $t) -> Measurement {
                Measurement::Int(value as i64)
            }
        }
    );

    ($t:ty, Float) => (
        impl From<$t> for Measurement {
            fn from(value: $t) -> Measurement {
                Measurement::Float(value as f64)
            }
        }
    )
}

measurement_from!(usize, Int);
measurement_from!(isize, Int);
measurement_from!(u8,  Int);
measurement_from!(i8,  Int);
measurement_from!(u16, Int);
measurement_from!(i16, Int);
measurement_from!(u32, Int);
measurement_from!(i32, Int);
measurement_from!(u64, Int);
measurement_from!(i64, Int);
measurement_from!(f32, Float);
measurement_from!(f64, Float);

#[derive(Clone, Copy, Debug)]
pub struct DurationMeasurement(Measurement);

impl fmt::Display for DurationMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for DurationMeasurement {
    fn from(value: u64) -> DurationMeasurement {
        DurationMeasurement(Measurement::Int(value as i64))
    }
}

impl From<chrono::Duration> for DurationMeasurement {
    fn from(value: chrono::Duration) -> DurationMeasurement {
        DurationMeasurement(Measurement::Int(value.num_milliseconds()))
    }
}

impl<'a> From<&'a chrono::Duration> for DurationMeasurement {
    fn from(value: &'a chrono::Duration) -> DurationMeasurement {
        DurationMeasurement(Measurement::Int(value.num_milliseconds()))
    }
}

#[derive(Clone, Debug)]
pub struct SetMeasurement<'a>(Cow<'a, str>);

impl<'a> fmt::Display for SetMeasurement<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> From<&'a str> for SetMeasurement<'a> {
    fn from(value: &'a str) -> SetMeasurement<'a> {
        SetMeasurement(Cow::Borrowed(value))
    }
}

impl<'a> From<u64> for SetMeasurement<'a> {
    fn from(value: u64) -> SetMeasurement<'a> {
        SetMeasurement(Cow::Owned(value.to_string()))
    }
}

/// The different states a service can be in
#[derive(Clone, Copy, Debug)]
pub enum ServiceStatus {
    /// OK State
    OK = 0,
    /// Warning State
    Warning = 1,
    /// Critical State
    Critical = 2,
    /// Unknown State
    Unknown = 3,
}

/// A UTC timestamp
#[derive(Clone, Copy, Debug)]
pub struct Timestamp(i64);

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for Timestamp {
    fn from(value: i64) -> Timestamp {
        Timestamp(value)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Timestamp {
        Timestamp(value.timestamp())
    }
}

impl<'a> From<&'a chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(value: &'a chrono::DateTime<chrono::Utc>) -> Timestamp {
        Timestamp(value.timestamp())
    }
}

/// Struct for adding optional pieces to a service check
#[derive(Clone, Copy, Debug, Default)]
pub struct ServiceCheckOptions<'a> {
    /// An optional timestamp to include with the check
    pub timestamp: Option<Timestamp>,
    /// An optional hostname to include with the check
    pub hostname: Option<&'a str>,
    /// An optional message to include with the check
    pub message: Option<&'a str>,
}

impl<'a> ServiceCheckOptions<'a> {
    fn len(&self) -> usize {
        let mut length = 0;
        length += self.timestamp.map_or(0, |ts| format!("{}", ts).len() + 3);
        length += self.hostname.map_or(0, |host| host.len() + 3);
        length += self.message.map_or(0, |msg| msg.len() + 3);
        length
    }
}

/// Struct for adding optional pieces for an event
#[derive(Clone, Copy, Debug, Default)]
pub struct EventOptions<'a> {
    pub timestamp: Option<Timestamp>,
    pub hostname: Option<&'a str>,
    pub aggregation_key: Option<&'a str>,
    pub priority: Option<EventPriority>,
    pub source_type: Option<&'a str>,
    pub alert_type: Option<EventAlertType>,
}

#[derive(Clone, Copy, Debug)]
pub enum EventPriority {
    Normal,
    Low,
}

impl fmt::Display for EventPriority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EventPriority::*;
        write!(f, "{}", match self {
            Normal => "normal",
            Low => "low",
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EventAlertType {
    Info,
    Error,
    Warning,
    Success,
}

impl fmt::Display for EventAlertType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EventAlertType::*;
        write!(f, "{}", match self {
            Info => "info",
            Error => "error",
            Warning => "warning",
            Success => "success",
        })
    }
}

pub fn format_for_send<I>(in_metric: &Metric, in_namespace: &str, tags: I) -> Vec<u8>
    where I: IntoIterator, I::Item: AsRef<str>
{
    let metric = in_metric.metric_type_format();
    let namespace = if in_metric.uses_namespace() {
        in_namespace
    } else {
        ""
    };
    let mut buf = Vec::with_capacity(metric.len() + namespace.len());

    if !namespace.is_empty() {
        buf.extend_from_slice(namespace.as_bytes());
        buf.extend_from_slice(b".");
    }

    buf.extend_from_slice(metric.as_bytes());

    let mut tags_iter = tags.into_iter();
    let mut next_tag = tags_iter.next();

    if next_tag.is_some() {
        buf.extend_from_slice(b"|#");
    }

    while next_tag.is_some() {
        buf.extend_from_slice(next_tag.unwrap().as_ref().as_bytes());

        next_tag = tags_iter.next();

        if next_tag.is_some() {
            buf.extend_from_slice(b",");
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use super::*;

    #[test]
    fn test_format_for_send_no_tags() {
        assert_eq!(
            &b"namespace.foo:1|c"[..],
            &format_for_send(&Metric::Count { stat: "foo", val: 1.into() }, "namespace", &[] as &[String])[..]
        )
    }

    #[test]
    fn test_format_for_send_no_namespace() {
        assert_eq!(
            &b"foo:1|c|#tag:1,tag:2"[..],
            &format_for_send(&Metric::Count { stat: "foo", val: 1.into() }, "", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_format_for_send_everything() {
        assert_eq!(
            &b"namespace.foo:1|c|#tag:1,tag:2"[..],
            &format_for_send(&Metric::Count { stat: "foo", val: 1.into() }, "namespace", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_format_for_send_everything_omit_namespace() {
        assert_eq!(
            &b"_e{5,4}:title|text|#tag:1,tag:2"[..],
            &format_for_send(&Metric::Event { title: "title", text: "text", opt: EventOptions::default() }, "namespace", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_count_incr_metric() {
        let metric = Metric::Count { stat: "incr", val: 1.into() };

        assert_eq!("incr:1|c", metric.metric_type_format())
    }

    #[test]
    fn test_count_decr_metric() {
        let metric = Metric::Count { stat: "decr", val: (-1).into() };

        assert_eq!("decr:-1|c", metric.metric_type_format())
    }

    #[test]
    fn test_time_metric() {
        let start_time = Utc.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 0);
        let end_time = Utc.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 900);
        let duration = end_time.signed_duration_since(start_time);
        let metric = Metric::Timing { stat: "time", val: duration.into() };

        assert_eq!("time:900|ms", metric.metric_type_format())
    }

    #[test]
    fn test_timing_metric() {
        let metric = Metric::Timing { stat: "timing", val: 720.into() };

        assert_eq!("timing:720|ms", metric.metric_type_format())
    }

    #[test]
    fn test_gauge_metric() {
        let metric = Metric::Gauge { stat: "gauge", val: 12345.into() };

        assert_eq!("gauge:12345|g", metric.metric_type_format())
    }

    #[test]
    fn test_histogram_metric() {
        let metric = Metric::Histogram { stat: "histogram", val: 67890.into() };

        assert_eq!("histogram:67890|h", metric.metric_type_format())
    }

    #[test]
    fn test_distribution_metric() {
        let metric = Metric::Distribution { stat: "distribution", val: 67890.into() };

        assert_eq!("distribution:67890|d", metric.metric_type_format())
    }

    #[test]
    fn test_set_metric() {
        let metric = Metric::Set { stat: "set", val: 13579.into() };
        assert_eq!("set:13579|s", metric.metric_type_format());

        let metric = Metric::Set { stat: "set", val: "13579".into() };
        assert_eq!("set:13579|s", metric.metric_type_format());
    }

    #[test]
    fn test_service_check() {
        let metric = Metric::ServiceCheck { stat: "redis.can_connect", status: ServiceStatus::Warning, opt: ServiceCheckOptions::default() };

        assert_eq!("_sc|redis.can_connect|1", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_timestamp() {
        let opt = ServiceCheckOptions {
            timestamp: Some(1234567890.into()),
            ..Default::default()
        };
        let metric = Metric::ServiceCheck { stat: "redis.can_connect", status: ServiceStatus::Warning, opt };

        assert_eq!("_sc|redis.can_connect|1|d:1234567890", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_hostname() {
        let opt = ServiceCheckOptions {
            hostname: Some("my_server.localhost"),
            ..Default::default()
        };
        let metric = Metric::ServiceCheck { stat: "redis.can_connect", status: ServiceStatus::Warning, opt };

        assert_eq!("_sc|redis.can_connect|1|h:my_server.localhost", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_message() {
        let opt = ServiceCheckOptions {
            message: Some("Service is possibly down"),
            ..Default::default()
        };
        let metric = Metric::ServiceCheck { stat: "redis.can_connect", status: ServiceStatus::Warning, opt };

        assert_eq!("_sc|redis.can_connect|1|m:Service is possibly down", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_all() {
        let opt = ServiceCheckOptions {
            timestamp: Some(1234567890.into()),
            hostname: Some("my_server.localhost"),
            message: Some("Service is possibly down")
        };
        let metric = Metric::ServiceCheck { stat: "redis.can_connect", status: ServiceStatus::Warning, opt };

        assert_eq!(
            "_sc|redis.can_connect|1|d:1234567890|h:my_server.localhost|m:Service is possibly down",
            metric.metric_type_format()
        )
    }

    #[test]
    fn test_event() {
        let metric = Metric::Event { 
            title: "Event Title",
            text: "Event Body - Something Happened",
            opt: EventOptions::default() 
        };
        assert_eq!(
            "_e{11,31}:Event Title|Event Body - Something Happened",
            metric.metric_type_format()
        );

        let metric = Metric::Event { 
            title: "Event Title",
            text: "Event Body - Something Happened",
            opt: EventOptions {
                timestamp: Some(1234567890.into()),
                hostname: Some("localhost"),
                aggregation_key: Some("key"),
                priority: Some(EventPriority::Low),
                source_type: Some("alert source"),
                alert_type: Some(EventAlertType::Warning),
            }
        };
        assert_eq!(
            "_e{11,31}:Event Title|Event Body - Something Happened|d:1234567890|h:localhost|k:key|p:low|s:alert source|t:warning",
            metric.metric_type_format()
        );
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    extern crate test;

    use self::test::Bencher;
    use super::*;

    #[bench]
    fn bench_format_for_send(b: &mut Bencher) {
        b.iter(|| {
            format_for_send("metric", "foo", &["bar", "baz"]);
        })
    }


    #[bench]
    fn bench_set_metric(b: &mut Bencher) {
        let metric = SetMetric {
            stat: "blahblahblah-blahblahblah",
            val: "valuel"
        };

        b.iter(|| {
            metric.metric_type_format()
        })
    }

    #[bench]
    fn bench_set_counter(b: &mut Bencher) {
        let metric = CountMetric::Incr("foo");

        b.iter(|| {
            metric.metric_type_format()
        })
    }
}
