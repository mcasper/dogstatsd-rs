use chrono::{DateTime, Utc};

pub fn format_for_send<I, S>(metric: &str, namespace: &str, tags: I) -> Vec<u8>
    where I: IntoIterator<Item=S>,
          S: AsRef<str>,
{
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
        buf.push_str(":");
        buf.push_str(&dur.num_milliseconds().to_string());
        buf.push_str("|ms");
        buf
    }
}

impl<'a> TimeMetric<'a> {
    pub fn new(stat: &'a str, start_time: &'a DateTime<Utc>, end_time: &'a DateTime<Utc>) -> Self {
        TimeMetric {
            start_time: start_time,
            end_time: end_time,
            stat: stat,
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
        buf.push_str(":");
        buf.push_str(&ms);
        buf.push_str("|ms");
        buf
    }
}

impl<'a> TimingMetric<'a> {
    pub fn new(stat: &'a str, ms: i64) -> Self {
        TimingMetric {
            ms: ms,
            stat: stat,
        }
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
        buf.push_str(":");
        buf.push_str(self.val);
        buf.push_str("|g");
        buf
    }
}

impl<'a> GaugeMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        GaugeMetric {
            stat: stat,
            val: val,
        }
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
        buf.push_str(":");
        buf.push_str(self.val);
        buf.push_str("|h");
        buf
    }
}

impl<'a> HistogramMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        HistogramMetric {
            stat: stat,
            val: val,
        }
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
        buf.push_str(":");
        buf.push_str(self.val);
        buf.push_str("|s");
        buf
    }
}

impl<'a> SetMetric<'a> {
    pub fn new(stat: &'a str, val: &'a str) -> Self {
        SetMetric {
            stat: stat,
            val: val,
        }
    }
}

pub struct Event<'a> {
    title: &'a str,
    text: &'a str,
}

impl<'a> Metric for Event<'a> {
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
            &b"namespace.metric:val|v"[..],
            &format_for_send("metric:val|v", "namespace", &[] as &[String])[..]
        )
    }

    #[test]
    fn test_format_for_send_no_namespace() {
        assert_eq!(
            &b"metric:val|v|#tag:1,tag:2"[..],
            &format_for_send("metric:val|v", "", &["tag:1", "tag:2"])[..]
        )
    }

    #[test]
    fn test_format_for_send_everything() {
        assert_eq!(
            &b"namespace.metric:val|v|#tag:1,tag:2"[..],
            &format_for_send("metric:val|v", "namespace", &["tag:1", "tag:2"])[..]
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
        let start_time = Utc.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 0);
        let end_time = Utc.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 900);
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
    fn test_set_metric() {
        let metric = SetMetric::new("set".into(), "13579".into());

        assert_eq!("set:13579|s", metric.metric_type_format())
    }

    #[test]
    fn test_event() {
        let metric = Event::new("Event Title".into(), "Event Body - Something Happened".into());

        assert_eq!("_e{11,31}:Event Title|Event Body - Something Happened",
                   metric.metric_type_format())
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
