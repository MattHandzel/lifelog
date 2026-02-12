#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use harness::simulated_modalities::{payload_matches_modality, simulated_all_modality_chunks};
use harness::TestContext;
use lifelog_types::{GetDataRequest, Query, QueryRequest};

#[tokio::test]
async fn test_all_modalities_simulated_dataflow_roundtrip() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "sim-all-modalities";
    let session_id = 777u64;
    let base = chrono::Utc::now() - chrono::Duration::seconds(5);

    let cases = simulated_all_modality_chunks(collector_id, session_id, base);

    // Upload each modality payload through the same collector->server path.
    for case in &cases {
        let stream = tokio_stream::iter(vec![case.chunk.clone()]);
        let _ack = client
            .upload_chunks(stream)
            .await
            .expect("upload chunk should succeed")
            .into_inner();
    }

    // Allow DB/search indexes to settle.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let start =
        (base - chrono::Duration::seconds(10)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let end =
        (base + chrono::Duration::seconds(20)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    for case in &cases {
        let llql = format!(
            r#"llql:{{
              "target": {{"type":"modality","modality":"{}"}},
              "filter": {{"op":"time_range","start":"{}","end":"{}"}}
            }}"#,
            case.modality.as_str_name(),
            start,
            end
        );

        let query = Query {
            text: vec![llql],
            ..Default::default()
        };

        let resp = client
            .query(QueryRequest { query: Some(query) })
            .await
            .expect("query should succeed")
            .into_inner();

        let key = resp
            .keys
            .iter()
            .find(|k| k.uuid == case.uuid)
            .cloned()
            .unwrap_or_else(|| {
                panic!(
                    "expected uuid {} for stream {} modality {}",
                    case.uuid,
                    case.stream_id,
                    case.modality.as_str_name()
                )
            });

        let data = client
            .get_data(GetDataRequest { keys: vec![key] })
            .await
            .expect("get_data should succeed")
            .into_inner();

        assert_eq!(data.data.len(), 1, "expected one payload per modality");
        assert!(
            payload_matches_modality(case.modality, &data.data[0]),
            "payload variant mismatch for modality {}",
            case.modality.as_str_name()
        );
    }
}
