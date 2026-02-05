use crate::chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeInterval {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>, // half-open [start, end)
}

impl TimeInterval {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Option<Self> {
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub fn overlaps(self, other: Self) -> bool {
        core::cmp::max(self.start, other.start) < core::cmp::min(self.end, other.end)
    }

    pub fn contains_point(self, t: DateTime<Utc>) -> bool {
        self.start <= t && t < self.end
    }
}

/// WITHIN for two point times.
pub fn within(a: DateTime<Utc>, b: DateTime<Utc>, delta: Duration) -> bool {
    let dt = a - b;
    dt.num_milliseconds().abs() <= delta.num_milliseconds()
}

/// WITHIN for a point time and an interval: true if the point is within `delta` of the interval.
pub fn within_interval(t: DateTime<Utc>, interval: TimeInterval, delta: Duration) -> bool {
    if interval.contains_point(t) {
        return true;
    }
    let before = t < interval.start && (interval.start - t) <= delta;
    let after = t >= interval.end && (t - interval.end) <= delta;
    before || after
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrono::TimeZone;

    #[test]
    fn within_uses_delta() {
        let t0 = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let t1 = t0 + Duration::seconds(29);
        assert!(within(t0, t1, Duration::seconds(30)));
        assert!(!within(t0, t1, Duration::seconds(10)));
    }

    #[test]
    fn overlaps_matches_worked_example() {
        let base = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let a = TimeInterval::new(base, base + Duration::minutes(5)).unwrap();
        let b = TimeInterval::new(base + Duration::minutes(4), base + Duration::minutes(10)).unwrap();
        assert!(a.overlaps(b));
    }
}

