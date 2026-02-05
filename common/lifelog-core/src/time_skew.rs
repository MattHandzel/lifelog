use crate::chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeQuality {
    Unknown,
    Good,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkewEstimate {
    /// Estimated offset such that `t_canonical = t_device + offset`.
    pub offset: Duration,
    /// 0.0..=1.0 confidence score. Higher is better.
    pub confidence: f32,
    pub time_quality: TimeQuality,
}

impl SkewEstimate {
    pub fn apply(&self, t_device: DateTime<Utc>) -> DateTime<Utc> {
        t_device + self.offset
    }
}

/// Estimate clock skew from samples of `(device_now, backend_now)`.
///
/// This is intentionally simple and deterministic:
/// - offset candidates are `(backend_now - device_now)`
/// - estimate is the median offset
/// - confidence is derived from median absolute deviation (MAD)
pub fn estimate_skew(samples: &[(DateTime<Utc>, DateTime<Utc>)]) -> SkewEstimate {
    if samples.is_empty() {
        return SkewEstimate {
            offset: Duration::zero(),
            confidence: 0.0,
            time_quality: TimeQuality::Unknown,
        };
    }

    let mut offsets: Vec<i64> = samples
        .iter()
        .map(|(device_now, backend_now)| (*backend_now - *device_now).num_milliseconds())
        .collect();
    offsets.sort_unstable();
    let median_ms = median_i64(&offsets);

    // Median absolute deviation as a robust jitter measure.
    let mut abs_devs: Vec<i64> = offsets
        .iter()
        .map(|v| (v - median_ms).abs())
        .collect();
    abs_devs.sort_unstable();
    let mad_ms = median_i64(&abs_devs);

    // Heuristic: <50ms is "very stable", 50..500ms degrades gradually, above that is low.
    let confidence = if mad_ms <= 50 {
        0.95
    } else if mad_ms >= 5_000 {
        0.05
    } else {
        let t = (mad_ms as f32 - 50.0) / (5_000.0 - 50.0);
        // Linear falloff in (0.05..0.95]
        (0.95 - 0.9 * t).clamp(0.05, 0.95)
    };

    let time_quality = if confidence >= 0.85 {
        TimeQuality::Good
    } else if confidence >= 0.4 {
        TimeQuality::Degraded
    } else {
        TimeQuality::Unknown
    };

    SkewEstimate {
        offset: Duration::milliseconds(median_ms),
        confidence,
        time_quality,
    }
}

fn median_i64(sorted: &[i64]) -> i64 {
    debug_assert!(!sorted.is_empty());
    sorted[sorted.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrono::TimeZone;

    #[test]
    fn estimate_skew_empty_is_unknown() {
        let est = estimate_skew(&[]);
        assert_eq!(est.time_quality, TimeQuality::Unknown);
        assert_eq!(est.confidence, 0.0);
        assert_eq!(est.offset, Duration::zero());
    }

    #[test]
    fn estimate_skew_stable_samples_high_confidence() {
        let d0 = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let b0 = d0 + Duration::seconds(5);
        let samples = vec![
            (d0, b0),
            (d0 + Duration::seconds(10), b0 + Duration::seconds(10)),
            (d0 + Duration::seconds(20), b0 + Duration::seconds(20)),
        ];

        let est = estimate_skew(&samples);
        assert_eq!(est.offset, Duration::seconds(5));
        assert!(est.confidence > 0.8, "confidence = {}", est.confidence);
        assert_eq!(est.time_quality, TimeQuality::Good);
        assert_eq!(est.apply(d0), b0);
    }

    #[test]
    fn estimate_skew_jitter_degrades_confidence() {
        let d0 = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let samples = vec![
            (d0, d0 + Duration::seconds(5)),
            (d0 + Duration::seconds(10), d0 + Duration::seconds(10) + Duration::seconds(6)),
            (d0 + Duration::seconds(20), d0 + Duration::seconds(20) + Duration::seconds(4)),
        ];
        let est = estimate_skew(&samples);
        assert_eq!(est.offset, Duration::seconds(5));
        assert!(est.confidence < 0.95);
        assert_ne!(est.time_quality, TimeQuality::Good);
    }
}

