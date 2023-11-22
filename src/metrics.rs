use chrono::{DateTime, Utc};

pub fn format_for_send<M, I, S>(
    in_metric: &M,
    in_namespace: &str,
    tags: I,
    default_tags: &Vec<u8>,
) -> Vec<u8>
where
    M: Metric,
    I: IntoIterator<Item = S>,
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
    let has_tags = next_tag.is_some();

    if next_tag.is_some() || !default_tags.is_empty() {
        buf.extend_from_slice(b"|#");
    }

    while next_tag.is_some() {
        buf.extend_from_slice(next_tag.unwrap().as_ref().as_bytes());

        next_tag = tags_iter.next();

        if next_tag.is_some() {
            buf.extend_from_slice(b",");
        }
    }

    if !default_tags.is_empty() {
        if has_tags {
            buf.extend_from_slice(b",")
        }

        buf.extend_from_slice(default_tags);
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
    Incr(&'a str, i64),
    Decr(&'a str, i64),
    Arbitrary(&'a str, i64),
}

impl<'a> Metric for CountMetric<'a> {
    // my_count:1|c
    // my_count:-1|c
    fn metric_type_format(&self) -> String {
        match *self {
            CountMetric::Incr(stat, amount) => {
                let mut buf = String::with_capacity(3 + stat.len() + amount.to_string().len() + 3);
                buf.push_str(stat);
                buf.push_str(&format!(":{}|c", amount));
                buf
            }
            CountMetric::Decr(stat, amount) => {
                let mut buf = String::with_capacity(3 + stat.len() + amount.to_string().len() + 4);
                buf.push_str(stat);
                buf.push_str(&format!(":{}|c", -amount));
                buf
            }
            CountMetric::Arbitrary(stat, amount) => {
                let mut buf = String::with_capacity(3 + stat.len() + 23);
                buf.push_str(stat);
                buf.push(':');
                buf.push_str(&amount.to_string());
                buf.push_str("|c");
                buf
            }
        }
    }
}

pub struct TimeMetric<'a> {
    start_time: &'a DateTime<Utc>,
    end_time: &'a DateTime<Utc>,
    stat: &'a str,
}

impl<'a> Metric for TimeMetric<'a> {
    // my_stat:500|ms
    fn metric_type_format(&self) -> String {
        let dur = self.end_time.signed_duration_since(*self.start_time);
        let mut buf = String::with_capacity(3 + self.stat.len() + 11);
        buf.push_str(self.stat);
        buf.push(':');
        buf.push_str(&dur.num_milliseconds().to_string());
        buf.push_str("|ms");
        buf
    }
}

impl<'a> TimeMetric<'a> {
    pub fn new(stat: &'a str, start_time: &'a DateTime<Utc>, end_time: &'a DateTime<Utc>) -> Self {
        TimeMetric {
            start_time,
            end_time,
            stat,
        }
    }
}

pub struct TimingMetric<'a> {
    ms: i64,
    stat: &'a str,
}

impl<'a> Metric for TimingMetric<'a> {
    // my_stat:500|ms
    fn metric_type_format(&self) -> String {
        let ms = self.ms.to_string();
        let mut buf = String::with_capacity(3 + self.stat.len() + ms.len());
        buf.push_str(self.stat);
        buf.push(':');
        buf.push_str(&ms);
        buf.push_str("|ms");
        buf
    }
}

impl<'a> TimingMetric<'a> {
    pub fn new(stat: &'a str, ms: i64) -> Self {
        TimingMetric { ms, stat }
    }
}

pub struct GaugeMetric<'a> {
    stat: &'a str,
    val: &'a str,
}

impl<'a> Metric for GaugeMetric<'a> {
    // my_gauge:1000|g
    fn metric_type_format(&self) -> String {
        let mut buf = String::with_capacity(3 + self.stat.len() + self.val.len());
        buf.push_str(self.stat);
        buf.push(':');
        buf.push_str(self.val);
        buf.push_str("|g");
        buf
    }
}

impl<'a> GaugeMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        GaugeMetric { stat, val }
    }
}

pub struct HistogramMetric<'a> {
    stat: &'a str,
    val: &'a str,
}

impl<'a> Metric for HistogramMetric<'a> {
    // my_histogram:1000|h
    fn metric_type_format(&self) -> String {
        let mut buf = String::with_capacity(3 + self.stat.len() + self.val.len());
        buf.push_str(self.stat);
        buf.push(':');
        buf.push_str(self.val);
        buf.push_str("|h");
        buf
    }
}

impl<'a> HistogramMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        HistogramMetric { stat, val }
    }
}

pub struct DistributionMetric<'a> {
    stat: &'a str,
    val: &'a str,
}

impl<'a> Metric for DistributionMetric<'a> {
    // my_distribution:1000|d
    fn metric_type_format(&self) -> String {
        let mut buf = String::with_capacity(3 + self.stat.len() + self.val.len());
        buf.push_str(self.stat);
        buf.push(':');
        buf.push_str(self.val);
        buf.push_str("|d");
        buf
    }
}

impl<'a> DistributionMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        DistributionMetric { stat, val }
    }
}

pub struct SetMetric<'a> {
    stat: &'a str,
    val: &'a str,
}

impl<'a> Metric for SetMetric<'a> {
    // my_set:45|s
    fn metric_type_format(&self) -> String {
        let mut buf = String::with_capacity(3 + self.stat.len() + self.val.len());
        buf.push_str(self.stat);
        buf.push(':');
        buf.push_str(self.val);
        buf.push_str("|s");
        buf
    }
}

impl<'a> SetMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        SetMetric { stat, val }
    }
}

/// Represents the different states a service can be in
#[derive(Clone, Copy, Debug)]
pub enum ServiceStatus {
    /// OK State
    OK,
    /// Warning State
    Warning,
    /// Critical State
    Critical,
    /// Unknown State
    Unknown,
}

impl ServiceStatus {
    fn to_int(self) -> i32 {
        match self {
            ServiceStatus::OK => 0,
            ServiceStatus::Warning => 1,
            ServiceStatus::Critical => 2,
            ServiceStatus::Unknown => 3,
        }
    }
}

/// Struct for adding optional pieces to a service check
#[derive(Default, Clone, Copy, Debug)]
pub struct ServiceCheckOptions<'a> {
    /// An optional timestamp to include with the check
    pub timestamp: Option<i32>,
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

pub struct ServiceCheck<'a> {
    stat: &'a str,
    val: ServiceStatus,
    options: ServiceCheckOptions<'a>,
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
        buf.push('|');
        buf.push_str(&format!("{}", self.val.to_int()));

        if self.options.timestamp.is_some() {
            buf.push_str("|d:");
            buf.push_str(&format!("{}", self.options.timestamp.unwrap()));
        }

        if self.options.hostname.is_some() {
            buf.push_str("|h:");
            buf.push_str(self.options.hostname.unwrap());
        }

        if self.options.message.is_some() {
            buf.push_str("|m:");
            buf.push_str(self.options.message.unwrap());
        }

        buf
    }
}

impl<'a> ServiceCheck<'a> {
    pub fn new(stat: &'a str, val: ServiceStatus, options: ServiceCheckOptions<'a>) -> Self {
        ServiceCheck { stat, val, options }
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
        let mut buf = String::with_capacity(
            self.title.len() + self.text.len() + title_len.len() + text_len.len() + 6,
        );
        buf.push_str("_e{");
        buf.push_str(&title_len);
        buf.push(',');
        buf.push_str(&text_len);
        buf.push_str("}:");
        buf.push_str(self.title);
        buf.push('|');
        buf.push_str(self.text);
        buf
    }
}

impl<'a> Event<'a> {
    pub fn new(title: &'a str, text: &'a str) -> Self {
        Event { title, text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_format_for_send_no_tags() {
        assert_eq!(
            &b"namespace.foo:1|c"[..],
            &format_for_send(
                &CountMetric::Incr("foo", 1),
                "namespace",
                &[] as &[String],
                &String::default().into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_format_for_optional_value_in_increment() {
        assert_eq!(
            &b"namespace.foo:20|c"[..],
            &format_for_send(
                &CountMetric::Incr("foo", 20),
                "namespace",
                &[] as &[String],
                &String::default().into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_format_for_send_no_namespace() {
        assert_eq!(
            &b"foo:1|c|#tag:1,tag:2"[..],
            &format_for_send(
                &CountMetric::Incr("foo", 1),
                "",
                &["tag:1", "tag:2"],
                &String::default().into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_format_for_no_default_tags() {
        assert_eq!(
            &b"namespace.foo:1|c|#tag:1,tag:2,defaultag:3,seconddefault:4"[..],
            &format_for_send(
                &CountMetric::Incr("foo", 1),
                "namespace",
                &["tag:1", "tag:2"],
                &String::from("defaultag:3,seconddefault:4").into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_format_for_send_everything() {
        assert_eq!(
            &b"namespace.foo:1|c|#tag:1,tag:2,defaultag:3,seconddefault:4"[..],
            &format_for_send(
                &CountMetric::Incr("foo", 1),
                "namespace",
                &["tag:1", "tag:2"],
                &String::from("defaultag:3,seconddefault:4").into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_format_for_send_everything_omit_namespace() {
        assert_eq!(
            &b"_e{5,4}:title|text|#tag:1,tag:2"[..],
            &format_for_send(
                &Event::new("title".into(), "text".into()),
                "namespace",
                &["tag:1", "tag:2"],
                &String::default().into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_format_with_only_default_tags() {
        assert_eq!(
            &b"namespace.foo:1|c|#defaultag:3,seconddefault:4"[..],
            &format_for_send(
                &CountMetric::Incr("foo", 1),
                "namespace",
                &[] as &[String],
                &String::from("defaultag:3,seconddefault:4").into_bytes()
            )[..]
        )
    }

    #[test]
    fn test_count_incr_metric() {
        let metric = CountMetric::Incr("incr".into(), 1);

        assert_eq!("incr:1|c", metric.metric_type_format())
    }

    #[test]
    fn test_count_decr_metric() {
        let metric = CountMetric::Decr("decr".into(), 1);

        assert_eq!("decr:-1|c", metric.metric_type_format())
    }

    #[test]
    fn test_count_decr_by_value_metric() {
        let metric = CountMetric::Decr("decr".into(), 35);

        assert_eq!("decr:-35|c", metric.metric_type_format())
    }

    #[test]
    fn test_count_metric() {
        let metric = CountMetric::Arbitrary("arb".into(), 54321);
        assert_eq!("arb:54321|c", metric.metric_type_format());
        let metric = CountMetric::Arbitrary("arb".into(), -12345);
        assert_eq!("arb:-12345|c", metric.metric_type_format());
        let metric = CountMetric::Arbitrary("arb".into(), 0);
        assert_eq!("arb:0|c", metric.metric_type_format());
    }

    #[test]
    fn test_time_metric() {
        let start_time = Utc.with_ymd_and_hms(2016, 4, 24, 0, 0, 0).unwrap();
        let end_time = Utc
            .timestamp_millis_opt(start_time.timestamp_millis() + 900)
            .unwrap();
        let metric = TimeMetric::new("time".into(), &start_time, &end_time);

        assert_eq!("time:900|ms", metric.metric_type_format())
    }

    #[test]
    fn test_timing_metric() {
        let metric = TimingMetric::new("timing".into(), 720);

        assert_eq!("timing:720|ms", metric.metric_type_format())
    }

    #[test]
    fn test_gauge_metric() {
        let metric = GaugeMetric::new("gauge".into(), "12345".into());

        assert_eq!("gauge:12345|g", metric.metric_type_format())
    }

    #[test]
    fn test_histogram_metric() {
        let metric = HistogramMetric::new("histogram".into(), "67890".into());

        assert_eq!("histogram:67890|h", metric.metric_type_format())
    }

    #[test]
    fn test_distribution_metric() {
        let metric = DistributionMetric::new("distribution".into(), "67890".into());

        assert_eq!("distribution:67890|d", metric.metric_type_format())
    }

    #[test]
    fn test_set_metric() {
        let metric = SetMetric::new("set".into(), "13579".into());

        assert_eq!("set:13579|s", metric.metric_type_format())
    }

    #[test]
    fn test_service_check() {
        let metric = ServiceCheck::new(
            "redis.can_connect".into(),
            ServiceStatus::Warning,
            ServiceCheckOptions::default(),
        );

        assert_eq!("_sc|redis.can_connect|1", metric.metric_type_format())
    }

    #[test]
    fn test_service_check_with_timestamp() {
        let options = ServiceCheckOptions {
            timestamp: Some(1234567890),
            ..Default::default()
        };
        let metric = ServiceCheck::new("redis.can_connect".into(), ServiceStatus::Warning, options);

        assert_eq!(
            "_sc|redis.can_connect|1|d:1234567890",
            metric.metric_type_format()
        )
    }

    #[test]
    fn test_service_check_with_hostname() {
        let options = ServiceCheckOptions {
            hostname: Some("my_server.localhost"),
            ..Default::default()
        };
        let metric = ServiceCheck::new("redis.can_connect".into(), ServiceStatus::Warning, options);

        assert_eq!(
            "_sc|redis.can_connect|1|h:my_server.localhost",
            metric.metric_type_format()
        )
    }

    #[test]
    fn test_service_check_with_message() {
        let options = ServiceCheckOptions {
            message: Some("Service is possibly down"),
            ..Default::default()
        };
        let metric = ServiceCheck::new("redis.can_connect".into(), ServiceStatus::Warning, options);

        assert_eq!(
            "_sc|redis.can_connect|1|m:Service is possibly down",
            metric.metric_type_format()
        )
    }

    #[test]
    fn test_service_check_with_all() {
        let options = ServiceCheckOptions {
            timestamp: Some(1234567890),
            hostname: Some("my_server.localhost"),
            message: Some("Service is possibly down"),
        };
        let metric = ServiceCheck::new("redis.can_connect".into(), ServiceStatus::Warning, options);

        assert_eq!(
            "_sc|redis.can_connect|1|d:1234567890|h:my_server.localhost|m:Service is possibly down",
            metric.metric_type_format()
        )
    }

    #[test]
    fn test_event() {
        let metric = Event::new(
            "Event Title".into(),
            "Event Body - Something Happened".into(),
        );

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

    struct NullMetric;

    impl Metric for NullMetric {
        fn metric_type_format(&self) -> String {
            String::new()
        }
    }

    #[bench]
    fn bench_format_for_send(b: &mut Bencher) {
        let metric = NullMetric;

        b.iter(|| {
            format_for_send(
                &metric,
                "foo",
                &["bar", "baz"],
                &String::default().into_bytes(),
            );
        })
    }

    #[bench]
    fn bench_set_metric(b: &mut Bencher) {
        let metric = SetMetric {
            stat: "blahblahblah-blahblahblah",
            val: "valuel",
        };

        b.iter(|| metric.metric_type_format())
    }

    #[bench]
    fn bench_set_counter(b: &mut Bencher) {
        let metric = CountMetric::Incr("foo", 1);

        b.iter(|| metric.metric_type_format())
    }
}
