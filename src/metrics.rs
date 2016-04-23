use chrono::{DateTime, UTC};

pub trait Metric {
    fn format_for_send(&self) -> String;
}

pub enum CountMetric {
    Incr(String),
    Decr(String),
}

impl Metric for CountMetric {
    // my_count:1|c
    // my_count:-1|c
    fn format_for_send(&self) -> String {
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
    fn format_for_send(&self) -> String {
        let dur = self.end_time - self.start_time;
        format!("{}:{}|ms", self.stat, dur.num_milliseconds())
    }
}

impl TimeMetric {
    pub fn new(stat: &str, start_time: DateTime<UTC>, end_time: DateTime<UTC>) -> Self {
        TimeMetric {
            start_time: start_time,
            end_time: end_time,
            stat: stat.into(),
        }
    }
}

pub struct TimingMetric {
    ms: i64,
    stat: String,
}

impl Metric for TimingMetric {
    // my_stat:500|ms
    fn format_for_send(&self) -> String {
        format!("{}:{}|ms", self.stat, self.ms)
    }
}

impl TimingMetric {
    pub fn new(stat: &str, ms: i64) -> Self {
        TimingMetric {
            ms: ms,
            stat: stat.into(),
        }
    }
}

pub struct GaugeMetric {
    stat: String,
    val: String,
}

impl Metric for GaugeMetric {
    // my_gauge:1000|g
    fn format_for_send(&self) -> String {
        format!("{}:{}|g", self.stat, self.val)
    }
}

impl GaugeMetric {
    pub fn new(stat: &str, val: &str) -> Self {
        GaugeMetric {
            stat: stat.into(),
            val: val.into(),
        }
    }
}

pub struct HistogramMetric {
    stat: String,
    val: String,
}

impl Metric for HistogramMetric {
    // my_histogram:1000|h
    fn format_for_send(&self) -> String {
        format!("{}:{}|h", self.stat, self.val)
    }
}

impl HistogramMetric {
    pub fn new(stat: &str, val: &str) -> Self {
        HistogramMetric {
            stat: stat.into(),
            val: val.into(),
        }
    }
}

pub struct SetMetric {
    stat: String,
    val: String,
}

impl Metric for SetMetric {
    // my_set:45|s
    fn format_for_send(&self) -> String {
        format!("{}:{}|s", self.stat, self.val)
    }
}

impl SetMetric {
    pub fn new(stat: &str, val: &str) -> Self {
        SetMetric {
            stat: stat.into(),
            val: val.into(),
        }
    }
}

pub struct Event {
    title: String,
    text: String,
}

impl Metric for Event {
    fn format_for_send(&self) -> String {
        format!("_e{{{title_len},{text_len}}}:{title}|{text}",
                title_len = self.title.len(),
                text_len = self.text.len(),
                title = self.title,
                text = self.text)
    }
}

impl Event {
    pub fn new(title: &str, text: &str) -> Self {
        Event {
            title: title.into(),
            text: text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, UTC};
    use super::*;

    #[test]
    fn test_count_incr_metric() {
        let metric = CountMetric::Incr("incr".into());

        assert_eq!("incr:1|c", metric.format_for_send())
    }

    #[test]
    fn test_count_decr_metric() {
        let metric = CountMetric::Decr("decr".into());

        assert_eq!("decr:-1|c", metric.format_for_send())
    }

    #[test]
    fn test_time_metric() {
        let start_time = UTC.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 0);
        let end_time = UTC.ymd(2016, 4, 24).and_hms_milli(0, 0, 0, 900);
        let metric = TimeMetric::new("time", start_time, end_time);

        assert_eq!("time:900|ms", metric.format_for_send())
    }

    #[test]
    fn test_timing_metric() {
        let metric = TimingMetric::new("timing", 720);

        assert_eq!("timing:720|ms", metric.format_for_send())
    }

    #[test]
    fn test_gauge_metric() {
        let metric = GaugeMetric::new("gauge", "12345");

        assert_eq!("gauge:12345|g", metric.format_for_send())
    }

    #[test]
    fn test_histogram_metric() {
        let metric = HistogramMetric::new("histogram", "67890");

        assert_eq!("histogram:67890|h", metric.format_for_send())
    }

    #[test]
    fn test_set_metric() {
        let metric = SetMetric::new("set", "13579");

        assert_eq!("set:13579|s", metric.format_for_send())
    }

    #[test]
    fn test_event() {
        let metric = Event::new("Event Title", "Event Body - Something Happened");

        assert_eq!("_e{11,31}:Event Title|Event Body - Something Happened",
                   metric.format_for_send())
    }
}
