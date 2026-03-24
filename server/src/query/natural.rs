use chrono::{Duration, Local, NaiveTime, TimeZone, Utc};

use super::ast::{Expression, Query, StreamSelector};

pub fn parse_natural_query(input: &str) -> Query {
    let input = input.trim();
    if input.is_empty() {
        return Query {
            target: StreamSelector::All,
            filter: Expression::TimeRange(Utc::now() - Duration::hours(24), Utc::now()),
        };
    }

    let (time_expr, remaining) = extract_time_expression(input);
    let (modality, remaining) = extract_modality_hint(&remaining);
    let target = modality
        .map(|m| StreamSelector::Modality(m.to_string()))
        .unwrap_or(StreamSelector::All);

    let keywords = remaining.trim();

    let mut filters: Vec<Expression> = Vec::new();

    if let Some(time) = time_expr {
        filters.push(time);
    }

    if !keywords.is_empty() {
        filters.push(Expression::Contains(
            "text".to_string(),
            keywords.to_string(),
        ));
    }

    let filter = if filters.is_empty() {
        Expression::TimeRange(Utc::now() - Duration::hours(24), Utc::now())
    } else {
        filters
            .into_iter()
            .reduce(|a, b| Expression::And(Box::new(a), Box::new(b)))
            .unwrap()
    };

    Query { target, filter }
}

fn extract_time_expression(input: &str) -> (Option<Expression>, String) {
    let lower = input.to_lowercase();
    let now = Utc::now();

    let time_patterns: &[(
        &str,
        Box<dyn Fn() -> (chrono::DateTime<Utc>, chrono::DateTime<Utc>)>,
    )] = &[
        (
            "today",
            Box::new(move || {
                let start = Local::now()
                    .date_naive()
                    .and_time(NaiveTime::MIN)
                    .and_local_timezone(Local)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now - Duration::hours(24));
                (start, now)
            }),
        ),
        (
            "yesterday",
            Box::new(move || {
                let yesterday = Local::now().date_naive() - Duration::days(1);
                let start = yesterday
                    .and_time(NaiveTime::MIN)
                    .and_local_timezone(Local)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now - Duration::hours(48));
                let end = (yesterday + Duration::days(1))
                    .and_time(NaiveTime::MIN)
                    .and_local_timezone(Local)
                    .single()
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now - Duration::hours(24));
                (start, end)
            }),
        ),
        (
            "last hour",
            Box::new(move || (now - Duration::hours(1), now)),
        ),
        (
            "last 24 hours",
            Box::new(move || (now - Duration::hours(24), now)),
        ),
        (
            "last week",
            Box::new(move || (now - Duration::weeks(1), now)),
        ),
        (
            "this week",
            Box::new(move || (now - Duration::weeks(1), now)),
        ),
        (
            "last 30 minutes",
            Box::new(move || (now - Duration::minutes(30), now)),
        ),
        (
            "this morning",
            Box::new(move || {
                let start = Local::now()
                    .date_naive()
                    .and_hms_opt(6, 0, 0)
                    .and_then(|dt| Local.from_local_datetime(&dt).single())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now - Duration::hours(12));
                let end = Local::now()
                    .date_naive()
                    .and_hms_opt(12, 0, 0)
                    .and_then(|dt| Local.from_local_datetime(&dt).single())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now);
                (start, end)
            }),
        ),
    ];

    for (pattern, range_fn) in time_patterns {
        if let Some(pos) = lower.find(pattern) {
            let (start, end) = range_fn();
            let remaining = format!("{} {}", &input[..pos], &input[pos + pattern.len()..]);
            return (Some(Expression::TimeRange(start, end)), remaining);
        }
    }

    (None, input.to_string())
}

fn extract_modality_hint(input: &str) -> (Option<&'static str>, String) {
    let lower = input.to_lowercase();
    let modality_map: &[(&[&str], &str)] = &[
        (&["screenshots", "screenshot", "screen"], "Screen"),
        (&["browser", "browsing", "web"], "Browser"),
        (&["audio", "sound", "recording"], "Audio"),
        (&["clipboard", "copied", "pasted"], "Clipboard"),
        (&["keystrokes", "keystroke", "typing", "typed"], "Keystroke"),
        (&["shell", "terminal", "command"], "ShellHistory"),
        (&["window", "windows", "app"], "WindowActivity"),
        (&["ocr", "text recognition"], "Ocr"),
        (&["camera", "webcam", "photo"], "Camera"),
        (&["transcription", "transcript"], "Transcription"),
        (&["weather"], "Weather"),
    ];

    for (keywords, modality) in modality_map {
        for kw in *keywords {
            if let Some(pos) = lower.find(kw) {
                let before = if pos > 0 {
                    input[pos - 1..pos]
                        .chars()
                        .next()
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false)
                } else {
                    false
                };
                let after_pos = pos + kw.len();
                let after = if after_pos < input.len() {
                    input[after_pos..after_pos + 1]
                        .chars()
                        .next()
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false)
                } else {
                    false
                };

                if !before && !after {
                    let remaining = format!("{} {}", &input[..pos], &input[after_pos..]);
                    return (Some(modality), remaining);
                }
            }
        }
    }

    (None, input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_keyword_only() {
        let q = parse_natural_query("rust programming");
        assert_eq!(q.target, StreamSelector::All);
        assert!(
            matches!(q.filter, Expression::Contains(_, ref text) if text == "rust programming")
        );
    }

    #[test]
    fn parses_modality_hint() {
        let q = parse_natural_query("browser youtube");
        assert_eq!(q.target, StreamSelector::Modality("Browser".to_string()));
        assert!(matches!(q.filter, Expression::Contains(_, ref text) if text.trim() == "youtube"));
    }

    #[test]
    fn parses_time_expression() {
        let q = parse_natural_query("last hour");
        assert_eq!(q.target, StreamSelector::All);
        assert!(matches!(q.filter, Expression::TimeRange(_, _)));
    }

    #[test]
    fn parses_combined() {
        let q = parse_natural_query("screenshots today rust");
        assert_eq!(q.target, StreamSelector::Modality("Screen".to_string()));
        assert!(matches!(q.filter, Expression::And(_, _)));
    }

    #[test]
    fn empty_input_defaults() {
        let q = parse_natural_query("");
        assert_eq!(q.target, StreamSelector::All);
        assert!(matches!(q.filter, Expression::TimeRange(_, _)));
    }
}
