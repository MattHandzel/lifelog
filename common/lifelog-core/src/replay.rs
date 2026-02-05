use crate::chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayStep {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Map screen point frames into replay intervals:
/// - frames are point timestamps
/// - steps are `[t_i, t_{i+1})`, with the last step ending at `window_end`
pub fn build_replay_steps(mut frame_times: Vec<DateTime<Utc>>, window_end: DateTime<Utc>) -> Vec<ReplayStep> {
    frame_times.sort();
    frame_times.dedup();

    match frame_times.len() {
        0 => Vec::new(),
        1 => {
            let t0 = frame_times[0];
            if t0 >= window_end {
                Vec::new()
            } else {
                vec![ReplayStep { start: t0, end: window_end }]
            }
        }
        _ => {
            let mut steps = Vec::with_capacity(frame_times.len());
            for pair in frame_times.windows(2) {
                let start = pair[0];
                let end = pair[1];
                if start < end {
                    steps.push(ReplayStep { start, end });
                }
            }

            // Extend final step to window_end if needed.
            if let Some(last_t) = frame_times.last().copied() {
                if let Some(prev) = frame_times.get(frame_times.len() - 2).copied() {
                    if last_t > prev && last_t < window_end {
                        steps.push(ReplayStep {
                            start: last_t,
                            end: window_end,
                        });
                    }
                }
            }

            steps
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chrono::{Duration, TimeZone};

    #[test]
    fn replay_steps_multi_frame_maps_to_intervals() {
        let t0 = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let t1 = t0 + Duration::seconds(10);
        let t2 = t0 + Duration::seconds(20);

        let steps = build_replay_steps(vec![t0, t1, t2], t2);
        assert_eq!(
            steps,
            vec![
                ReplayStep { start: t0, end: t1 },
                ReplayStep { start: t1, end: t2 },
            ]
        );
    }

    #[test]
    fn replay_steps_single_frame_extends_to_window_end() {
        let t0 = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let end = t0 + Duration::seconds(30);

        let steps = build_replay_steps(vec![t0], end);
        assert_eq!(steps, vec![ReplayStep { start: t0, end }]);
    }
}

