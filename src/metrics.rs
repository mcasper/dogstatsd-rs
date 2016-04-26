use chrono::{DateTime, UTC};

pub fn format_for_send(metric: String, namespace: &String, tags: Vec<String>) -> String {
    let mut result = metric;

    if namespace != "" {
        result = format!("{}.{}", namespace, result)
    }

    let joined_tags = tags.join(",");
    if joined_tags != "" {
        result = format!("{}|{}", result, joined_tags)
    }

    result
}

pub trait Metric {
    fn metric_type_format(&self) -> String;
}

pub enum CountMetric {
    Incr(String),
    Decr(String),
}

impl Metric for CountMetric {
    // my_count:1|c
    // my_count:-1|c
    fn metric_type_format(&self) -> String {
        match *self {
            CountMetric::Incr(ref stat) => {
                format!("{}:1|c", stat)
            },
            CountMetric::Decr(ref stat) => {
                format!("{}:-1|c", stat)
            },
        }
    }
}

pub struct TimeMetric {
    start_time: DateTime<UTC>,
    end_time: DateTime<UTC>,
    stat: String,
}

impl Metric for TimeMetric {
    // my_stat:500|ms
    fn metric_type_format(&self) -> String {
        let dur = self.end_time - self.start_time;
        format!("{}:{}|ms", self.stat, dur.num_milliseconds())
    }
}

impl TimeMetric {
    pub fn new(stat: String, start_time: DateTime<UTC>, end_time: DateTime<UTC>) -> Self {
        TimeMetric {
            start_time: start_time,
            end_time: end_time,
            stat: stat,
        }
    }
}

pub struct TimingMetric {
    ms: i64,
    stat: String,
}

impl Metric for TimingMetric {
    // my_stat:500|ms
    fn metric_type_format(&self) -> String {
        format!("{}:{}|ms", self.stat, self.ms)
    }
}

impl TimingMetric {
    pub fn new(stat: String, ms: i64) -> Self {
        TimingMetric {
            ms: ms,
            stat: stat,
        }
    }
}

pub struct GaugeMetric {
    stat: String,
    val: String,
}

impl Metric for GaugeMetric {
    // my_gauge:1000|g
    fn metric_type_format(&self) -> String {
        format!("{}:{}|g", self.stat, self.val)
    }
}

impl GaugeMetric {
    pub fn new(stat: String, val: String) -> Self {
        GaugeMetric {
            stat: stat,
            val: val,
        }
    }
}

pub struct HistogramMetric {
    stat: String,
    val: String,
}

impl Metric for HistogramMetric {
    // my_histogram:1000|h
    fn metric_type_format(&self) -> String {
        format!("{}:{}|h", self.stat, self.val)
    }
}

impl HistogramMetric {
    pub fn new(stat: String, val: String) -> Self {
        HistogramMetric {
            stat: stat,
            val: val,
        }
    }
}

pub struct SetMetric {
    stat: String,
    val: String,
}

impl Metric for SetMetric {
    // my_set:45|s
    fn metric_type_format(&self) -> String {
        format!("{}:{}|s", self.stat, self.val)
    }
}

impl SetMetric {
    pub fn new(stat: String, val: String) -> Self {
        SetMetric {
            stat: stat,
            val: val,
        }
    }
}

pub struct Event {
    title: String,
    text: String,
}

impl Metric for Event {
    fn metric_type_format(&self) -> String {
        format!("_e{{{title_len},{text_len}}}:{title}|{text}",
                title_len = self.title.len(),
                text_len = self.text.len(),
                title = self.title,
                text = self.text)
    }
}

impl Event {
    pub fn new(title: String, text: String) -> Self {
        Event {
            title: title,
            text: text,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, UTC};
    use super::*;

    #[test]
    fn test_format_for_send_no_tags() {
        assert_eq!(
            "namespace.metric:val|v".to_string(),
            format_for_send("metric:val|v".to_string(), &"namespace".to_string(), vec![])
        )
    }

    #[test]
    fn test_format_for_send_no_namespace() {
        assert_eq!(
            "metric:val|v|tag:1,tag:2".to_string(),
            format_for_send("metric:val|v".to_string(), &String::new(), vec!["tag:1".into(), "tag:2".into()])
        )
    }

    #[test]
    fn test_format_for_send_everything() {
        assert_eq!(
            "namespace.metric:val|v|tag:1,tag:2".to_string(),
            format_for_send("metric:val|v".to_string(), &"namespace".to_string(), vec!["tag:1".into(), "tag:2".into()])
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
        let start_time = UTC.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 0);
        let end_time = UTC.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 900);
        let metric = TimeMetric::new("time".into(), start_time, end_time);

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
