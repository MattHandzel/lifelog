#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod harness;

use chrono::{Duration, Utc};
use harness::TestContext;
use lifelog_core::{DataOrigin, DataOriginType};
use lifelog_types::{AudioFrame, BrowserFrame, Query, QueryRequest};
use prost::Message;

#[tokio::test]
#[ignore = "integration test: requires SurrealDB"]
async fn test_llql_canonical_example_audio_during_youtube_and_3b1b() {
    let _ = tracing_subscriber::fmt::try_init();

    let ctx = TestContext::new().await;
    let mut client = ctx.client();

    let collector_id = "test-collector";
    let session_id = 42u64;

    let base = Utc::now() - Duration::minutes(5);

    // Ingest one AudioFrame interval [base, base+10s)
    let audio_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "audio".to_string(),
        session_id,
    };

    let audio_uuid = lifelog_core::Uuid::new_v4().to_string();
    let audio_ts = lifelog_types::to_pb_ts(base);
    let audio_end = lifelog_types::to_pb_ts(base + Duration::seconds(10));
    let audio = AudioFrame {
        uuid: audio_uuid.clone(),
        timestamp: audio_ts,
        audio_bytes: vec![1; 10],
        codec: "pcm".to_string(),
        sample_rate: 48_000,
        channels: 1,
        duration_secs: 10.0,
        t_device: audio_ts,
        t_canonical: audio_ts,
        t_end: audio_end,
        ..Default::default()
    };

    let mut audio_buf = Vec::new();
    audio.encode(&mut audio_buf).unwrap();
    let audio_chunk = lifelog_types::Chunk {
        stream: Some(audio_stream),
        offset: 0,
        data: audio_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let audio_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&audio_chunk.data),
        ..audio_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![audio_chunk]))
        .await
        .expect("Ingest audio failed");

    // Ingest one BrowserFrame point at base+6s matching "youtube".
    let browser_stream = lifelog_types::StreamIdentity {
        collector_id: collector_id.to_string(),
        stream_id: "browser".to_string(),
        session_id,
    };

    let browser_ts = lifelog_types::to_pb_ts(base + Duration::seconds(6));
    let browser = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: browser_ts,
        url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        title: "Some Video".to_string(),
        visit_count: 1,
        t_device: browser_ts,
        t_canonical: browser_ts,
        t_end: browser_ts,
        ..Default::default()
    };

    let mut browser_buf = Vec::new();
    browser.encode(&mut browser_buf).unwrap();
    let browser_chunk = lifelog_types::Chunk {
        stream: Some(browser_stream),
        offset: 0,
        data: browser_buf,
        hash: utils::cas::sha256_hex(&[]),
    };
    let browser_chunk = lifelog_types::Chunk {
        hash: utils::cas::sha256_hex(&browser_chunk.data),
        ..browser_chunk
    };

    client
        .upload_chunks(tokio_stream::iter(vec![browser_chunk]))
        .await
        .expect("Ingest browser failed");

    // Seed an OcrRecord directly (collector-derived origin) so LLQL can resolve modality "Ocr".
    // We create the table schema + catalog entry to mimic the transform pipeline output.
    let db = surrealdb::Surreal::new::<surrealdb::engine::remote::ws::Ws>(&ctx.db_addr)
        .await
        .expect("DB Connect failed");
    db.signin(surrealdb::opt::auth::Root {
        username: "root",
        password: "root",
    })
    .await
    .expect("DB Signin failed");
    db.use_ns("lifelog")
        .use_db("test_db")
        .await
        .expect("DB Select failed");

    let screen_origin = DataOrigin::new(
        DataOriginType::DeviceId(collector_id.to_string()),
        "Screen".to_string(),
    );
    let ocr_origin = DataOrigin::new(
        DataOriginType::DataOrigin(Box::new(screen_origin.clone())),
        "Ocr".to_string(),
    );
    let ocr_table = ocr_origin.get_table_name();

    // Text analyzer required for SEARCH indexes.
    db.query(
        "DEFINE ANALYZER IF NOT EXISTS lifelog_text TOKENIZERS blank, class FILTERS lowercase, ascii, snowball(english);",
    )
    .await
    .expect("analyzer ddl failed")
    .check()
    .expect("analyzer ddl check failed");

    // Create Ocr schema (mirrors server/src/schema.rs for DataModality::Ocr).
    let ocr_ddl = format!(
        r#"
        DEFINE TABLE `{}` SCHEMAFULL;
        DEFINE FIELD uuid      ON `{}` TYPE string;
        DEFINE FIELD timestamp ON `{}` TYPE datetime;
        DEFINE FIELD text      ON `{}` TYPE string;
        DEFINE FIELD t_ingest    ON `{}` TYPE option<datetime>;
        DEFINE FIELD t_canonical ON `{}` TYPE option<datetime>;
        DEFINE FIELD t_end       ON `{}` TYPE option<datetime>;
        DEFINE FIELD time_quality ON `{}` TYPE option<string>;
        DEFINE INDEX `{}_ts_idx` ON `{}` FIELDS timestamp;
        DEFINE INDEX `{}_tcanon_idx` ON `{}` FIELDS t_canonical;
        DEFINE INDEX `{}_tend_idx` ON `{}` FIELDS t_end;
        DEFINE INDEX `{}_text_search` ON `{}` FIELDS text SEARCH ANALYZER lifelog_text BM25;
        "#,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
        ocr_table,
    );

    db.query(ocr_ddl)
        .await
        .expect("ocr schema failed")
        .check()
        .expect("ocr schema check failed");

    // Register derived origin in catalog using the same representation as ensure_table_schema.
    // For derived origins, `origin` is the parent table name.
    let _ = db
        .query(
            "UPSERT catalog SET origin = $origin, modality = $modality WHERE origin = $origin AND modality = $modality",
        )
        .bind(("origin", screen_origin.get_table_name()))
        .bind(("modality", "Ocr".to_string()))
        .await
        .expect("catalog upsert failed");

    let ocr_uuid = lifelog_core::Uuid::new_v4().to_string();
    let ocr_ts = base + Duration::seconds(8);
    let ocr_record = lifelog_types::OcrRecord {
        uuid: ocr_uuid,
        timestamp: ocr_ts.into(),
        text: "Watching 3Blue1Brown".to_string(),
        t_ingest: Some(Utc::now().into()),
        t_canonical: Some(ocr_ts.into()),
        t_end: Some(ocr_ts.into()),
        time_quality: Some("good".to_string()),
    };

    let _created: Option<lifelog_types::OcrRecord> = db
        .upsert((&ocr_table, &ocr_record.uuid))
        .content(ocr_record)
        .await
        .expect("insert ocr record failed");

    // Give SurrealDB a moment to update search indexes.
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    // Execute the canonical cross-modal query via LLQL.
    let llql = r#"llql:{
      "target": {"type":"modality","modality":"Audio"},
      "filter": {"op":"and","terms":[
        {"op":"during","stream":{"type":"modality","modality":"Browser"},"predicate":{"op":"contains","field":"url","text":"youtube"},"window":"5s"},
        {"op":"during","stream":{"type":"modality","modality":"Ocr"},"predicate":{"op":"contains","field":"text","text":"3Blue1Brown"},"window":"5s"}
      ]}
    }"#
    .to_string();

    let query = Query {
        text: vec![llql],
        ..Default::default()
    };

    let resp = client
        .query(QueryRequest { query: Some(query) })
        .await
        .expect("LLQL query failed")
        .into_inner();

    let uuids: Vec<String> = resp.keys.iter().map(|k| k.uuid.to_string()).collect();
    assert!(
        uuids.contains(&audio_uuid),
        "expected audio frame to match canonical DURING(browser AND ocr) query"
    );
}
