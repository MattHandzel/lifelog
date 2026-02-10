use chrono::{DateTime, Duration, Utc};
use lifelog_core::{replay::build_replay_steps, DataOrigin, LifelogFrameKey};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct ReplayStepInternal {
    pub(crate) start: DateTime<Utc>,
    pub(crate) end: DateTime<Utc>,
    pub(crate) screen_key: LifelogFrameKey,
    pub(crate) context_keys: Vec<LifelogFrameKey>,
}

#[derive(Debug, Clone)]
pub(crate) struct IntervalKey {
    pub(crate) key: LifelogFrameKey,
    pub(crate) start: DateTime<Utc>,
    pub(crate) end: DateTime<Utc>,
}

fn dt_key(dt: DateTime<Utc>) -> (i64, u32) {
    (dt.timestamp(), dt.timestamp_subsec_nanos())
}

pub(crate) fn build_replay_steps_for_screen(
    mut screen_frames: Vec<(String, DateTime<Utc>)>,
    screen_origin: DataOrigin,
    window_end: DateTime<Utc>,
) -> Vec<ReplayStepInternal> {
    screen_frames.sort_by_key(|(_uuid, t)| *t);

    let mut time_to_uuid: HashMap<(i64, u32), String> = HashMap::new();
    let mut times = Vec::with_capacity(screen_frames.len());
    for (uuid, t) in screen_frames {
        times.push(t);
        time_to_uuid.insert(dt_key(t), uuid);
    }

    let steps = build_replay_steps(times, window_end);
    steps
        .into_iter()
        .filter_map(|s| {
            let uuid = time_to_uuid.get(&dt_key(s.start))?.clone();
            Some(ReplayStepInternal {
                start: s.start,
                end: s.end,
                screen_key: LifelogFrameKey {
                    uuid: uuid.parse().ok()?,
                    origin: screen_origin.clone(),
                },
                context_keys: Vec::new(),
            })
        })
        .collect()
}

pub(crate) fn assign_context_keys(
    steps: &mut [ReplayStepInternal],
    mut records: Vec<IntervalKey>,
    pad: Duration,
    max_per_step: usize,
) {
    if steps.is_empty() || records.is_empty() || max_per_step == 0 {
        return;
    }

    // Deterministic: sort by (start, uuid, origin).
    records.sort_by_key(|r| (r.start, r.key.uuid, r.key.origin.get_table_name()));

    let mut idx = 0usize;
    for step in steps.iter_mut() {
        if step.context_keys.len() >= max_per_step {
            continue;
        }

        let s0 = step.start - pad;
        let s1 = step.end + pad;

        while idx < records.len() && records[idx].end < s0 {
            idx += 1;
        }

        let mut j = idx;
        while j < records.len() && records[j].start <= s1 {
            let r = &records[j];
            // overlap: [r.start, r.end] intersects [s0, s1]
            if r.start <= s1 && r.end >= s0 {
                step.context_keys.push(r.key.clone());
                if step.context_keys.len() >= max_per_step {
                    break;
                }
            }
            j += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use lifelog_core::{DataOriginType, Uuid};

    #[test]
    fn assigns_overlapping_context_to_steps() {
        let screen_origin = DataOrigin::new(DataOriginType::DeviceId("d".into()), "Screen".into());
        let ctx_origin = DataOrigin::new(DataOriginType::DeviceId("d".into()), "Ocr".into());

        let t0 = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let t1 = t0 + chrono::Duration::seconds(10);
        let t2 = t0 + chrono::Duration::seconds(20);

        let screen_frames = vec![
            (Uuid::new_v4().to_string(), t0),
            (Uuid::new_v4().to_string(), t1),
            (Uuid::new_v4().to_string(), t2),
        ];

        let mut steps = build_replay_steps_for_screen(screen_frames, screen_origin, t2);
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].start, t0);
        assert_eq!(steps[0].end, t1);

        let r0 = IntervalKey {
            key: LifelogFrameKey {
                uuid: Uuid::new_v4(),
                origin: ctx_origin.clone(),
            },
            start: t0 + chrono::Duration::seconds(2),
            end: t0 + chrono::Duration::seconds(3),
        };
        let r1 = IntervalKey {
            key: LifelogFrameKey {
                uuid: Uuid::new_v4(),
                origin: ctx_origin,
            },
            start: t1 + chrono::Duration::seconds(1),
            end: t1 + chrono::Duration::seconds(1),
        };

        assign_context_keys(&mut steps, vec![r0, r1], chrono::Duration::seconds(0), 10);
        assert_eq!(steps[0].context_keys.len(), 1);
        assert_eq!(steps[1].context_keys.len(), 1);
    }
}
