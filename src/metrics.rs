use chrono::{DateTime, Utc};

pub fn format_for_send<M, I, S>(in_metric: &M, in_namespace: &str, tags: I) -> Vec<u8>
    where M: Metric,
          I: IntoIterator<Item=S>,
          S: AsRef<str>,
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

pub trait Metric {
    fn metric_type_format(&self) -> String;

    fn uses_namespace(&self) -> bool {
        true
    }
}

pub enum CountMetric<'a> {
    Incr(&'a str),
    Decr(&'a str),
}

impl<'a> Metric for CountMetric<'a> {
    // my_count:1|c
    // my_count:-1|c
    fn metric_type_format(&self) -> String {
        match *self {
            CountMetric::Incr(stat) => {
                let mut buf = String::with_capacity(3 + stat.len() + 4);
                buf.push_str(stat);
                buf.push_str(":1|c");
                buf
            },
            CountMetric::Decr(stat) => {
                let mut buf = String::with_capacity(3 + stat.len() + 4);
                buf.push_str(stat);
                buf.push_str(":-1|c");
                buf
            },
        }
    }
}

pub struct TimeMetric<'a> {
    pub start_time: &'a DateTime<Utc>,
    pub end_time: &'a DateTime<Utc>,
    pub stat: &'a str,
}

impl<'a> Metric for TimeMetric<'a> {
    // my_stat:500|ms
    fn metric_type_format(&self) -> String {
        let dur = self.end_time.signed_duration_since(*self.start_time);
        let mut buf = String::with_capacity(3 + self.stat.len() + 11);
        buf.push_str(self.stat);
        buf.push_str(":");
        buf.push_str(&dur.num_milliseconds().to_string());
        buf.push_str("|ms");
        buf
    }
}

pub struct TimingMetric<'a> {
    pub ms: i64,
    pub stat: &'a str,
}

impl<'a> Metric for TimingMetric<'a> {
    // my_stat:500|ms
    fn metric_type_format(&self) -> String {
        let ms = self.ms.to_string();
        let mut buf = String::with_capacity(3 + self.stat.len() + ms.len());
        buf.push_str(self.stat);
        buf.push_str(":");
        buf.push_str(&ms);
        buf.push_str("|ms");
        buf
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Measurement {
    Int(i64),
    Float(f64),
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

macro_rules! measurement_metric {
    ($t:ident, $suffix:expr) => (

        pub struct $t<'a> {
            pub stat: &'a str,
            pub val: Measurement,
        }
        
        impl<'a> Metric for $t<'a> {
            // my_gauge:1000|$suffix
            fn metric_type_format(&self) -> String {
                let val = match self.val {
                    Measurement::Int(i) => format!("{}", i),
                    Measurement::Float(f) => format!("{:.6}", f),
                };

                let mut buf = String::with_capacity(3 + self.stat.len() + val.len());
                buf.push_str(self.stat);
                buf.push_str(":");
                buf.push_str(&val);
                buf.push_str($suffix);
                buf
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

measurement_metric!(GaugeMetric, "|g");
measurement_metric!(HistogramMetric, "|h");
measurement_metric!(DistributionMetric, "|d");
measurement_metric!(SetMetric, "|s");

/// Represents the different states a service can be in
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

/// Struct for adding optional pieces to a service check
#[derive(Clone, Copy, Debug)]
pub struct ServiceCheckOptions {
    /// An optional timestamp to include with the check
    pub timestamp: Option<i32>,
    /// An optional hostname to include with the check
    pub hostname: Option<&'static str>,
    /// An optional message to include with the check
    pub message: Option<&'static str>,
}

impl Default for ServiceCheckOptions {
    fn default() -> Self {
        ServiceCheckOptions {
            timestamp: None,
            hostname: None,
            message: None,
        }
    }
}

impl ServiceCheckOptions {
    fn len(&self) -> usize {
        let mut length = 0;
        length += self.timestamp.map_or(0, |ts| format!("{}", ts).len() + 3);
        length += self.hostname.map_or(0, |host| host.len() + 3);
        length += self.message.map_or(0, |msg| msg.len() + 3);
        length
    }
}

pub struct ServiceCheck<'a> {
    pub stat: &'a str,
    pub val: ServiceStatus,
    pub options: ServiceCheckOptions,
}

impl<'a> Metric for ServiceCheck<'a> {
    fn uses_namespace(&self) -> bool {
        false
    }

    // _sc|my_service.can_connect|1
    fn metric_type_format(&self) -> String {
        let mut buf = String::with_capacity(6 + self.stat.len() + self.options.len());
        buf.push_str("_sc|");
        buf.push_str(self.stat);
        buf.push_str("|");
        buf.push_str(&format!("{}", self.val as u8));

        if let Some(timestamp) = self.options.timestamp {
            buf.push_str("|d:");
            buf.push_str(&format!("{}", timestamp));
        }

        if let Some(hostname) = self.options.hostname {
            buf.push_str("|h:");
            buf.push_str(hostname);
        }

        if let Some(message) = self.options.message {
            buf.push_str("|m:");
            buf.push_str(message);
        }

        buf
    }
}

pub struct Event<'a> {
    title: &'a str,
    text: &'a str,
}

impl<'a> Metric for Event<'a> {
    fn uses_namespace(&self) -> bool {
        false
    }

    fn metric_type_format(&self) -> String {
        let title_len = self.title.len().to_string();
        let text_len = self.text.len().to_string();
        let mut buf = String::with_capacity(self.title.len() + self.text.len() + title_len.len() + text_len.len() + 6);
        buf.push_str("_e{");
        buf.push_str(&title_len);
        buf.push_str(",");
        buf.push_str(&text_len);
        buf.push_str("}:");
        buf.push_str(self.title);
        buf.push_str("|");
        buf.push_str(self.text);
        buf
    }
}

impl<'a> Event<'a> {
    pub fn new(title: &'a str, text: &'a str) -> Self {
        Event {
            title: title,
            text: text,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use super::*;

    #[test]
    fn test_format_for_send_no_tags() {
        assert_eq!(
            &b"namespace.foo:1|c"[..],
            &format_for_send(&CountMetric::Incr("foo"), "namespace", &[] as &[String])[..]
        )
    }

    #[test]
    fn test_format_for_send_no_namespace() {
        assert_eq!(
            &b"foo:1|c|#tag:1,tag:2"[..],
            &format_for_send(&CountMetric::Incr("foo"), "", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_format_for_send_everything() {
        assert_eq!(
            &b"namespace.foo:1|c|#tag:1,tag:2"[..],
            &format_for_send(&CountMetric::Incr("foo"), "namespace", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_format_for_send_everything_omit_namespace() {
        assert_eq!(
            &b"_e{5,4}:title|text|#tag:1,tag:2"[..],
            &format_for_send(&Event::new("title".into(), "text".into()), "namespace", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_count_incr_metric() {
        let metric = CountMetric::Incr("incr".into());

        assert_eq!("incr:1|c", metric.metric_type_format())
    }

    #[test]
    fn test_count_decr_metric() {
        let metric = CountMetric::Decr("decr".into());

        assert_eq!("decr:-1|c", metric.metric_type_format())
    }

    #[test]
    fn test_time_metric() {
        let ref start_time = Utc.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 0);
        let ref end_time = Utc.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 900);
        let metric = TimeMetric { stat: "time".into(), start_time, end_time };

        assert_eq!("time:900|ms", metric.metric_type_format())
    }

    #[test]
    fn test_timing_metric() {
        let metric = TimingMetric { stat: "timing".into(), ms: 720 };

        assert_eq!("timing:720|ms", metric.metric_type_format())
    }

    #[test]
    fn test_gauge_metric() {
        let metric = GaugeMetric { stat: "gauge".into(), val: 12345.into() };

        assert_eq!("gauge:12345|g", metric.metric_type_format())
    }

    #[test]
    fn test_histogram_metric() {
        let metric = HistogramMetric { stat: "histogram".into(), val: 67890.into() };

        assert_eq!("histogram:67890|h", metric.metric_type_format())
    }

    #[test]
    fn test_distribution_metric() {
        let metric = DistributionMetric { stat: "distribution".into(), val: 67890.into() };

        assert_eq!("distribution:67890|d", metric.metric_type_format())
    }

    #[test]
    fn test_set_metric() {
        let metric = SetMetric { stat: "set".into(), val: 13579.into() };

        assert_eq!("set:13579|s", metric.metric_type_format())
    }

    #[test]
    fn test_service_check() {
        let metric = ServiceCheck { stat: "redis.can_connect".into(), val: ServiceStatus::Warning, options: ServiceCheckOptions::default() };

        assert_eq!("_sc|redis.can_connect|1", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_timestamp() {
        let options = ServiceCheckOptions {
            timestamp: Some(1234567890),
            ..Default::default()
        };
        let metric = ServiceCheck { stat: "redis.can_connect".into(), val: ServiceStatus::Warning, options };

        assert_eq!("_sc|redis.can_connect|1|d:1234567890", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_hostname() {
        let options = ServiceCheckOptions {
            hostname: Some("my_server.localhost"),
            ..Default::default()
        };
        let metric = ServiceCheck { stat: "redis.can_connect".into(), val: ServiceStatus::Warning, options };

        assert_eq!("_sc|redis.can_connect|1|h:my_server.localhost", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_message() {
        let options = ServiceCheckOptions {
            message: Some("Service is possibly down"),
            ..Default::default()
        };
        let metric = ServiceCheck { stat: "redis.can_connect".into(), val: ServiceStatus::Warning, options };

        assert_eq!("_sc|redis.can_connect|1|m:Service is possibly down", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_all() {
        let options = ServiceCheckOptions {
            timestamp: Some(1234567890),
            hostname: Some("my_server.localhost"),
            message: Some("Service is possibly down")
        };
        let metric = ServiceCheck { stat: "redis.can_connect".into(), val: ServiceStatus::Warning, options };

        assert_eq!(
            "_sc|redis.can_connect|1|d:1234567890|h:my_server.localhost|m:Service is possibly down",
            metric.metric_type_format()
        )
    }

    #[test]
    fn test_event() {
        let metric = Event::new("Event Title".into(), "Event Body - Something Happened".into());

        assert_eq!(
            "_e{11,31}:Event Title|Event Body - Something Happened",
            metric.metric_type_format()
        )
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
