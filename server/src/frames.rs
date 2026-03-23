use chrono::{DateTime, Utc};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::postgres::PostgresPool;
use lifelog_core::{DataOrigin, DataOriginType, LifelogError, LifelogFrameKey};
use utils::cas::FsCas;

#[derive(Debug, Clone)]
pub struct FrameRow {
    pub id: Uuid,
    pub collector_id: String,
    pub stream_id: String,
    pub modality: String,
    pub t_device: Option<DateTime<Utc>>,
    pub t_ingest: DateTime<Utc>,
    pub t_canonical: DateTime<Utc>,
    pub t_end: Option<DateTime<Utc>>,
    pub time_quality: String,
    pub blob_hash: Option<String>,
    pub blob_size: Option<i32>,
    pub indexed: bool,
    pub source_frame_id: Option<Uuid>,
    pub payload: JsonValue,
}

impl FrameRow {
    pub fn insert_sql() -> &'static str {
        "INSERT INTO frames (
            id, collector_id, stream_id, modality, time_range,
            t_device, t_ingest, t_canonical, t_end, time_quality,
            blob_hash, blob_size, indexed, source_frame_id, payload
        ) VALUES (
            $1, $2, $3, $4, tstzrange($5, $6, '[]'),
            $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16
        )
        ON CONFLICT (id) DO NOTHING"
    }

    pub fn insert_params(&self) -> [&(dyn tokio_postgres::types::ToSql + Sync); 16] {
        let range_start = &self.t_canonical;
        let range_end = &self.t_end;
        [
            &self.id,
            &self.collector_id,
            &self.stream_id,
            &self.modality,
            range_start,
            range_end.as_ref().unwrap_or(range_start),
            &self.t_device,
            &self.t_ingest,
            &self.t_canonical,
            &self.t_end,
            &self.time_quality,
            &self.blob_hash,
            &self.blob_size,
            &self.indexed,
            &self.source_frame_id,
            &self.payload,
        ]
    }
}

fn pb_to_dt(ts: Option<pbjson_types::Timestamp>) -> DateTime<Utc> {
    let ts = ts.unwrap_or_default();
    DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_else(|| {
        DateTime::<Utc>::from_naive_utc_and_offset(chrono::NaiveDateTime::MIN, Utc)
    })
}

fn dt_to_pb(dt: DateTime<Utc>) -> pbjson_types::Timestamp {
    pbjson_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn parse_uuid(s: &str) -> Uuid {
    Uuid::parse_str(s).unwrap_or_default()
}

pub fn from_screen(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::ScreenFrame,
    cas: &FsCas,
) -> Result<FrameRow, String> {
    if frame.image_bytes.is_empty() {
        return Err("screen frame has empty image_bytes".to_string());
    }
    let blob_hash = cas
        .put(&frame.image_bytes)
        .map_err(|e| format!("CAS put failed: {e}"))?;
    let blob_size = frame.image_bytes.len() as i32;
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));

    Ok(FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Screen".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: Some(blob_hash),
        blob_size: Some(blob_size),
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "width": frame.width,
            "height": frame.height,
            "mime_type": frame.mime_type,
        }),
    })
}

pub fn from_browser(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::BrowserFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Browser".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "url": frame.url,
            "title": frame.title,
            "visit_count": frame.visit_count,
        }),
    }
}

pub fn from_audio(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::AudioFrame,
    cas: &FsCas,
) -> Result<FrameRow, String> {
    let blob_hash = cas
        .put(&frame.audio_bytes)
        .map_err(|e| format!("CAS put failed: {e}"))?;
    let blob_size = frame.audio_bytes.len() as i32;
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));

    Ok(FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Audio".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: Some(blob_hash),
        blob_size: Some(blob_size),
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "codec": frame.codec,
            "sample_rate": frame.sample_rate,
            "channels": frame.channels,
            "duration_secs": frame.duration_secs,
        }),
    })
}

pub fn from_clipboard(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::ClipboardFrame,
    cas: &FsCas,
) -> Result<FrameRow, String> {
    let (blob_hash, blob_size) = if !frame.binary_data.is_empty() {
        let hash = cas
            .put(&frame.binary_data)
            .map_err(|e| format!("CAS put failed: {e}"))?;
        let size = frame.binary_data.len() as i32;
        (Some(hash), Some(size))
    } else {
        (None, None)
    };
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));

    Ok(FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Clipboard".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash,
        blob_size,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "text": frame.text,
            "mime_type": frame.mime_type,
        }),
    })
}

pub fn from_shell_history(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::ShellHistoryFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "ShellHistory".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "command": frame.command,
            "working_dir": frame.working_dir,
            "exit_code": frame.exit_code,
        }),
    }
}

pub fn from_keystroke(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::KeystrokeFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Keystrokes".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "text": frame.text,
            "application": frame.application,
            "window_title": frame.window_title,
        }),
    }
}

pub fn from_mouse(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::MouseFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Mouse".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "x": frame.x,
            "y": frame.y,
            "button": frame.button,
            "pressed": frame.pressed,
        }),
    }
}

pub fn from_window_activity(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::WindowActivityFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "WindowActivity".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "application": frame.application,
            "window_title": frame.window_title,
            "focused": frame.focused,
            "duration_secs": frame.duration_secs,
        }),
    }
}

pub fn from_process(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::ProcessFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    let processes: Vec<JsonValue> = frame
        .processes
        .iter()
        .map(|p| {
            json!({
                "pid": p.pid,
                "ppid": p.ppid,
                "name": p.name,
                "exe": p.exe,
                "cmdline": p.cmdline,
                "status": p.status,
                "cpu_usage": p.cpu_usage,
                "memory_usage": p.memory_usage,
                "threads": p.threads,
                "user": p.user,
                "start_time": p.start_time,
            })
        })
        .collect();

    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Processes".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({ "processes": processes }),
    }
}

pub fn from_camera(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::CameraFrame,
    cas: &FsCas,
) -> Result<FrameRow, String> {
    let blob_hash = cas
        .put(&frame.image_bytes)
        .map_err(|e| format!("CAS put failed: {e}"))?;
    let blob_size = frame.image_bytes.len() as i32;
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));

    Ok(FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Camera".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: Some(blob_hash),
        blob_size: Some(blob_size),
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "width": frame.width,
            "height": frame.height,
            "mime_type": frame.mime_type,
            "device": frame.device,
        }),
    })
}

pub fn from_weather(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::WeatherFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Weather".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "temperature": frame.temperature,
            "humidity": frame.humidity,
            "pressure": frame.pressure,
            "conditions": frame.conditions,
        }),
    }
}

pub fn from_hyprland(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::HyprlandFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    let monitors: Vec<JsonValue> = frame
        .monitors
        .iter()
        .map(|m| {
            json!({
                "id": m.id, "name": m.name, "description": m.description,
                "width": m.width, "height": m.height, "refresh_rate": m.refresh_rate,
                "x": m.x, "y": m.y,
                "workspace_id": m.workspace_id, "workspace_name": m.workspace_name,
                "scale": m.scale, "focused": m.focused,
            })
        })
        .collect();
    let workspaces: Vec<JsonValue> = frame
        .workspaces
        .iter()
        .map(|w| {
            json!({
                "id": w.id, "name": w.name, "monitor": w.monitor,
                "monitor_id": w.monitor_id, "windows": w.windows,
                "fullscreen": w.fullscreen, "last_window": w.last_window,
                "last_window_title": w.last_window_title,
            })
        })
        .collect();
    let active_ws = frame.active_workspace.as_ref().map(|w| {
        json!({
            "id": w.id, "name": w.name, "monitor": w.monitor,
            "monitor_id": w.monitor_id, "windows": w.windows,
            "fullscreen": w.fullscreen, "last_window": w.last_window,
            "last_window_title": w.last_window_title,
        })
    });
    let clients: Vec<JsonValue> = frame
        .clients
        .iter()
        .map(|c| {
            json!({
                "address": c.address, "x": c.x, "y": c.y,
                "width": c.width, "height": c.height,
                "workspace_id": c.workspace_id, "workspace_name": c.workspace_name,
                "floating": c.floating, "fullscreen": c.fullscreen,
                "monitor": c.monitor, "title": c.title, "class": c.class,
                "pid": c.pid, "pinned": c.pinned, "mapped": c.mapped,
            })
        })
        .collect();
    let devices: Vec<JsonValue> = frame
        .devices
        .iter()
        .map(|d| {
            json!({
                "type": d.r#type, "name": d.name, "address": d.address,
            })
        })
        .collect();
    let cursor = frame.cursor.as_ref().map(|c| json!({"x": c.x, "y": c.y}));

    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Hyprland".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: None,
        payload: json!({
            "monitors": monitors,
            "workspaces": workspaces,
            "active_workspace": active_ws,
            "clients": clients,
            "devices": devices,
            "cursor": cursor,
        }),
    }
}

pub fn from_ocr(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::OcrFrame,
    source_frame_id: Option<Uuid>,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Ocr".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id,
        payload: json!({ "text": frame.text }),
    }
}

pub fn from_transcription(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::TranscriptionFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    let source = Uuid::parse_str(&frame.source_uuid).ok();
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "Transcription".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: source,
        payload: json!({
            "text": frame.text,
            "model": frame.model,
            "confidence": frame.confidence,
        }),
    }
}

pub fn from_embedding(
    collector_id: &str,
    stream_id: &str,
    frame: &lifelog_types::EmbeddingFrame,
) -> FrameRow {
    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
    let source = Uuid::parse_str(&frame.source_uuid).ok();
    FrameRow {
        id: parse_uuid(&frame.uuid),
        collector_id: collector_id.to_string(),
        stream_id: stream_id.to_string(),
        modality: "VectorEmbedding".to_string(),
        t_device: Some(t_device),
        t_ingest: Utc::now(),
        t_canonical: t_device,
        t_end: Some(t_device),
        time_quality: "unknown".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: true,
        source_frame_id: source,
        payload: json!({
            "model": frame.model,
            "dimensions": frame.dimensions,
            "vector": frame.vector,
        }),
    }
}

pub fn from_lifelog_data(
    collector_id: &str,
    stream_id: &str,
    data: &lifelog_types::LifelogData,
    cas: &FsCas,
) -> Result<FrameRow, String> {
    use lifelog_types::lifelog_data::Payload;
    match data.payload.as_ref() {
        Some(Payload::Screenframe(f)) => from_screen(collector_id, stream_id, f, cas),
        Some(Payload::Browserframe(f)) => Ok(from_browser(collector_id, stream_id, f)),
        Some(Payload::Audioframe(f)) => from_audio(collector_id, stream_id, f, cas),
        Some(Payload::Clipboardframe(f)) => from_clipboard(collector_id, stream_id, f, cas),
        Some(Payload::Shellhistoryframe(f)) => Ok(from_shell_history(collector_id, stream_id, f)),
        Some(Payload::Keystrokeframe(f)) => Ok(from_keystroke(collector_id, stream_id, f)),
        Some(Payload::Mouseframe(f)) => Ok(from_mouse(collector_id, stream_id, f)),
        Some(Payload::Windowactivityframe(f)) => {
            Ok(from_window_activity(collector_id, stream_id, f))
        }
        Some(Payload::Processframe(f)) => Ok(from_process(collector_id, stream_id, f)),
        Some(Payload::Cameraframe(f)) => from_camera(collector_id, stream_id, f, cas),
        Some(Payload::Weatherframe(f)) => Ok(from_weather(collector_id, stream_id, f)),
        Some(Payload::Hyprlandframe(f)) => Ok(from_hyprland(collector_id, stream_id, f)),
        Some(Payload::Ocrframe(f)) => Ok(from_ocr(collector_id, stream_id, f, None)),
        Some(Payload::Transcriptionframe(f)) => Ok(from_transcription(collector_id, stream_id, f)),
        Some(Payload::Embeddingframe(f)) => Ok(from_embedding(collector_id, stream_id, f)),
        None => Err("LifelogData has no payload".to_string()),
    }
}

pub fn to_lifelog_data(row: &FrameRow, cas: &FsCas) -> Result<lifelog_types::LifelogData, String> {
    use lifelog_types::lifelog_data::Payload;

    let t_device = row.t_device.map(dt_to_pb);
    let t_ingest = Some(dt_to_pb(row.t_ingest));
    let t_canonical = Some(dt_to_pb(row.t_canonical));
    let t_end = row.t_end.map(dt_to_pb);
    let timestamp = t_device.clone();
    let uuid = row.id.to_string();
    let p = &row.payload;

    let payload = match row.modality.as_str() {
        "Screen" => {
            let blob = load_blob(cas, &row.blob_hash)?;
            Payload::Screenframe(lifelog_types::ScreenFrame {
                uuid,
                timestamp,
                width: p["width"].as_u64().unwrap_or(0) as u32,
                height: p["height"].as_u64().unwrap_or(0) as u32,
                image_bytes: blob,
                mime_type: p["mime_type"].as_str().unwrap_or("").to_string(),
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        "Browser" => Payload::Browserframe(lifelog_types::BrowserFrame {
            uuid,
            timestamp,
            url: p["url"].as_str().unwrap_or("").to_string(),
            title: p["title"].as_str().unwrap_or("").to_string(),
            visit_count: p["visit_count"].as_u64().unwrap_or(0) as u32,
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "Audio" => {
            let blob = load_blob(cas, &row.blob_hash)?;
            Payload::Audioframe(lifelog_types::AudioFrame {
                uuid,
                timestamp,
                audio_bytes: blob,
                codec: p["codec"].as_str().unwrap_or("").to_string(),
                sample_rate: p["sample_rate"].as_u64().unwrap_or(0) as u32,
                channels: p["channels"].as_u64().unwrap_or(0) as u32,
                duration_secs: p["duration_secs"].as_f64().unwrap_or(0.0) as f32,
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        "Clipboard" => {
            let blob = match &row.blob_hash {
                Some(_) => load_blob(cas, &row.blob_hash)?,
                None => vec![],
            };
            Payload::Clipboardframe(lifelog_types::ClipboardFrame {
                uuid,
                timestamp,
                text: p["text"].as_str().unwrap_or("").to_string(),
                binary_data: blob,
                mime_type: p["mime_type"].as_str().unwrap_or("").to_string(),
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        "ShellHistory" => Payload::Shellhistoryframe(lifelog_types::ShellHistoryFrame {
            uuid,
            timestamp,
            command: p["command"].as_str().unwrap_or("").to_string(),
            working_dir: p["working_dir"].as_str().unwrap_or("").to_string(),
            exit_code: p["exit_code"].as_i64().unwrap_or(0) as i32,
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "Keystrokes" => Payload::Keystrokeframe(lifelog_types::KeystrokeFrame {
            uuid,
            timestamp,
            text: p["text"].as_str().unwrap_or("").to_string(),
            application: p["application"].as_str().unwrap_or("").to_string(),
            window_title: p["window_title"].as_str().unwrap_or("").to_string(),
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "Mouse" => Payload::Mouseframe(lifelog_types::MouseFrame {
            uuid,
            timestamp,
            x: p["x"].as_f64().unwrap_or(0.0),
            y: p["y"].as_f64().unwrap_or(0.0),
            button: p["button"].as_i64().unwrap_or(0) as i32,
            pressed: p["pressed"].as_bool().unwrap_or(false),
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "WindowActivity" => Payload::Windowactivityframe(lifelog_types::WindowActivityFrame {
            uuid,
            timestamp,
            application: p["application"].as_str().unwrap_or("").to_string(),
            window_title: p["window_title"].as_str().unwrap_or("").to_string(),
            focused: p["focused"].as_bool().unwrap_or(false),
            duration_secs: p["duration_secs"].as_f64().unwrap_or(0.0) as f32,
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "Processes" => {
            let procs = p["processes"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|pi| lifelog_types::ProcessInfo {
                            pid: pi["pid"].as_i64().unwrap_or(0) as i32,
                            ppid: pi["ppid"].as_i64().unwrap_or(0) as i32,
                            name: pi["name"].as_str().unwrap_or("").to_string(),
                            exe: pi["exe"].as_str().unwrap_or("").to_string(),
                            cmdline: pi["cmdline"].as_str().unwrap_or("").to_string(),
                            status: pi["status"].as_str().unwrap_or("").to_string(),
                            cpu_usage: pi["cpu_usage"].as_f64().unwrap_or(0.0),
                            memory_usage: pi["memory_usage"].as_i64().unwrap_or(0),
                            threads: pi["threads"].as_i64().unwrap_or(0) as i32,
                            user: pi["user"].as_str().unwrap_or("").to_string(),
                            start_time: pi["start_time"].as_f64().unwrap_or(0.0),
                        })
                        .collect()
                })
                .unwrap_or_default();
            Payload::Processframe(lifelog_types::ProcessFrame {
                uuid,
                timestamp,
                processes: procs,
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        "Camera" => {
            let blob = load_blob(cas, &row.blob_hash)?;
            Payload::Cameraframe(lifelog_types::CameraFrame {
                uuid,
                timestamp,
                width: p["width"].as_u64().unwrap_or(0) as u32,
                height: p["height"].as_u64().unwrap_or(0) as u32,
                image_bytes: blob,
                mime_type: p["mime_type"].as_str().unwrap_or("").to_string(),
                device: p["device"].as_str().unwrap_or("").to_string(),
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        "Weather" => Payload::Weatherframe(lifelog_types::WeatherFrame {
            uuid,
            timestamp,
            temperature: p["temperature"].as_f64().unwrap_or(0.0),
            humidity: p["humidity"].as_f64().unwrap_or(0.0),
            pressure: p["pressure"].as_f64().unwrap_or(0.0),
            conditions: p["conditions"].as_str().unwrap_or("").to_string(),
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "Hyprland" => {
            let monitors = json_to_hypr_monitors(p);
            let workspaces = json_to_hypr_workspaces(&p["workspaces"]);
            let active_workspace = p["active_workspace"]
                .as_object()
                .map(|_| json_to_hypr_workspace(&p["active_workspace"]));
            let clients = p["clients"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|c| lifelog_types::HyprClient {
                            address: c["address"].as_str().unwrap_or("").to_string(),
                            x: c["x"].as_i64().unwrap_or(0) as i32,
                            y: c["y"].as_i64().unwrap_or(0) as i32,
                            width: c["width"].as_i64().unwrap_or(0) as i32,
                            height: c["height"].as_i64().unwrap_or(0) as i32,
                            workspace_id: c["workspace_id"].as_i64().unwrap_or(0) as i32,
                            workspace_name: c["workspace_name"].as_str().unwrap_or("").to_string(),
                            floating: c["floating"].as_bool().unwrap_or(false),
                            fullscreen: c["fullscreen"].as_str().unwrap_or("").to_string(),
                            monitor: c["monitor"].as_i64().unwrap_or(0) as i32,
                            title: c["title"].as_str().unwrap_or("").to_string(),
                            class: c["class"].as_str().unwrap_or("").to_string(),
                            pid: c["pid"].as_i64().unwrap_or(0) as i32,
                            pinned: c["pinned"].as_bool().unwrap_or(false),
                            mapped: c["mapped"].as_bool().unwrap_or(false),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let devices = p["devices"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|d| lifelog_types::HyprDevice {
                            r#type: d["type"].as_str().unwrap_or("").to_string(),
                            name: d["name"].as_str().unwrap_or("").to_string(),
                            address: d["address"].as_str().unwrap_or("").to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            let cursor = p["cursor"].as_object().map(|_| lifelog_types::HyprCursor {
                x: p["cursor"]["x"].as_f64().unwrap_or(0.0),
                y: p["cursor"]["y"].as_f64().unwrap_or(0.0),
            });
            Payload::Hyprlandframe(lifelog_types::HyprlandFrame {
                uuid,
                timestamp,
                monitors,
                workspaces,
                active_workspace,
                clients,
                devices,
                cursor,
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        "Ocr" => Payload::Ocrframe(lifelog_types::OcrFrame {
            uuid,
            timestamp,
            text: p["text"].as_str().unwrap_or("").to_string(),
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "Transcription" => Payload::Transcriptionframe(lifelog_types::TranscriptionFrame {
            uuid,
            timestamp,
            text: p["text"].as_str().unwrap_or("").to_string(),
            source_uuid: row
                .source_frame_id
                .map(|u| u.to_string())
                .unwrap_or_default(),
            model: p["model"].as_str().unwrap_or("").to_string(),
            confidence: p["confidence"].as_f64().unwrap_or(0.0) as f32,
            t_device,
            t_ingest,
            t_canonical,
            t_end,
            time_quality: 0,
            record_type: 0,
        }),
        "VectorEmbedding" => {
            let vector = p["vector"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect()
                })
                .unwrap_or_default();
            Payload::Embeddingframe(lifelog_types::EmbeddingFrame {
                uuid,
                timestamp,
                source_uuid: row
                    .source_frame_id
                    .map(|u| u.to_string())
                    .unwrap_or_default(),
                model: p["model"].as_str().unwrap_or("").to_string(),
                vector,
                dimensions: p["dimensions"].as_u64().unwrap_or(0) as u32,
                t_device,
                t_ingest,
                t_canonical,
                t_end,
                time_quality: 0,
                record_type: 0,
            })
        }
        other => return Err(format!("unknown modality: {other}")),
    };

    Ok(lifelog_types::LifelogData {
        payload: Some(payload),
    })
}

fn load_blob(cas: &FsCas, blob_hash: &Option<String>) -> Result<Vec<u8>, String> {
    match blob_hash {
        Some(h) => cas.get(h).map_err(|e| format!("CAS get failed: {e}")),
        None => Err("expected blob_hash but found None".to_string()),
    }
}

fn json_to_hypr_monitors(p: &JsonValue) -> Vec<lifelog_types::HyprMonitor> {
    p["monitors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|m| lifelog_types::HyprMonitor {
                    id: m["id"].as_i64().unwrap_or(0) as i32,
                    name: m["name"].as_str().unwrap_or("").to_string(),
                    description: m["description"].as_str().unwrap_or("").to_string(),
                    width: m["width"].as_i64().unwrap_or(0) as i32,
                    height: m["height"].as_i64().unwrap_or(0) as i32,
                    refresh_rate: m["refresh_rate"].as_f64().unwrap_or(0.0) as f32,
                    x: m["x"].as_i64().unwrap_or(0) as i32,
                    y: m["y"].as_i64().unwrap_or(0) as i32,
                    workspace_id: m["workspace_id"].as_i64().unwrap_or(0) as i32,
                    workspace_name: m["workspace_name"].as_str().unwrap_or("").to_string(),
                    scale: m["scale"].as_f64().unwrap_or(1.0) as f32,
                    focused: m["focused"].as_bool().unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn json_to_hypr_workspace(v: &JsonValue) -> lifelog_types::HyprWorkspace {
    lifelog_types::HyprWorkspace {
        id: v["id"].as_i64().unwrap_or(0) as i32,
        name: v["name"].as_str().unwrap_or("").to_string(),
        monitor: v["monitor"].as_str().unwrap_or("").to_string(),
        monitor_id: v["monitor_id"].as_i64().unwrap_or(0) as i32,
        windows: v["windows"].as_i64().unwrap_or(0) as i32,
        fullscreen: v["fullscreen"].as_bool().unwrap_or(false),
        last_window: v["last_window"].as_str().unwrap_or("").to_string(),
        last_window_title: v["last_window_title"].as_str().unwrap_or("").to_string(),
    }
}

fn json_to_hypr_workspaces(v: &JsonValue) -> Vec<lifelog_types::HyprWorkspace> {
    v.as_array()
        .map(|arr| arr.iter().map(json_to_hypr_workspace).collect())
        .unwrap_or_default()
}

fn row_to_frame_row(row: &tokio_postgres::Row) -> Result<FrameRow, LifelogError> {
    let t_canonical: DateTime<Utc> = row.get("t_canonical");
    Ok(FrameRow {
        id: row.get("id"),
        collector_id: row.get("collector_id"),
        stream_id: row.get("stream_id"),
        modality: row.get("modality"),
        t_device: row.get("t_device"),
        t_ingest: row.get("t_ingest"),
        t_canonical,
        t_end: row.get("t_end"),
        time_quality: row.get("time_quality"),
        blob_hash: row.get("blob_hash"),
        blob_size: row.get("blob_size"),
        indexed: row.get("indexed"),
        source_frame_id: row.get("source_frame_id"),
        payload: row.get("payload"),
    })
}

pub async fn get_by_id(
    pool: &PostgresPool,
    cas: &FsCas,
    id: uuid::Uuid,
) -> Result<lifelog_types::LifelogData, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let row = client
        .query_opt(
            "SELECT id, collector_id, stream_id, modality, t_device, t_ingest, t_canonical, t_end,
                    time_quality, blob_hash, blob_size, indexed, source_frame_id, payload
             FROM frames WHERE id = $1",
            &[&id],
        )
        .await
        .map_err(|e| LifelogError::Database(format!("frames select: {e}")))?
        .ok_or_else(|| LifelogError::Database(format!("frame not found: {id}")))?;

    let frame_row = row_to_frame_row(&row)?;
    to_lifelog_data(&frame_row, cas)
        .map_err(|e| LifelogError::Database(format!("frame conversion: {e}")))
}

pub async fn get_keys_after(
    pool: &PostgresPool,
    origin: &DataOrigin,
    after: DateTime<Utc>,
    limit: usize,
) -> Result<Vec<LifelogFrameKey>, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let collector_id = extract_collector_id(origin);
    let modality = &origin.modality_name;

    let rows = if let Some(cid) = collector_id {
        client
            .query(
                "SELECT id FROM frames WHERE modality = $1 AND collector_id = $2 AND t_canonical > $3 ORDER BY t_canonical ASC LIMIT $4",
                &[&modality, &cid, &after, &(limit as i64)],
            )
            .await
    } else {
        client
            .query(
                "SELECT id FROM frames WHERE modality = $1 AND t_canonical > $2 ORDER BY t_canonical ASC LIMIT $3",
                &[&modality, &after, &(limit as i64)],
            )
            .await
    }
    .map_err(|e| LifelogError::Database(format!("frames keys query: {e}")))?;

    let keys = rows
        .iter()
        .filter_map(|row| {
            let uuid: uuid::Uuid = row.get(0);
            Some(LifelogFrameKey::new(
                lifelog_core::Uuid::from_bytes(*uuid.as_bytes()),
                origin.clone(),
            ))
        })
        .collect();

    Ok(keys)
}

pub async fn get_origins(pool: &PostgresPool) -> Result<Vec<DataOrigin>, LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let rows = client
        .query("SELECT DISTINCT collector_id, modality FROM frames", &[])
        .await
        .map_err(|e| LifelogError::Database(format!("frames origins query: {e}")))?;

    let origins = rows
        .iter()
        .map(|row| {
            let collector_id: String = row.get(0);
            let modality: String = row.get(1);
            DataOrigin::new(DataOriginType::DeviceId(collector_id), modality)
        })
        .collect();

    Ok(origins)
}

pub async fn upsert(pool: &PostgresPool, row: &FrameRow) -> Result<(), LifelogError> {
    let client = pool
        .get()
        .await
        .map_err(|e| LifelogError::Database(format!("pool: {e}")))?;

    let sql = "INSERT INTO frames (
            id, collector_id, stream_id, modality, time_range,
            t_device, t_ingest, t_canonical, t_end, time_quality,
            blob_hash, blob_size, indexed, source_frame_id, payload
        ) VALUES (
            $1, $2, $3, $4, tstzrange($5, $6, '[]'),
            $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16
        )
        ON CONFLICT (id) DO UPDATE SET
            payload = EXCLUDED.payload,
            t_ingest = EXCLUDED.t_ingest,
            t_canonical = EXCLUDED.t_canonical,
            t_end = EXCLUDED.t_end,
            time_quality = EXCLUDED.time_quality,
            indexed = EXCLUDED.indexed,
            time_range = EXCLUDED.time_range";

    let params = row.insert_params();
    client
        .execute(sql, &params)
        .await
        .map_err(|e| LifelogError::Database(format!("frames upsert: {e}")))?;

    Ok(())
}

fn extract_collector_id(origin: &DataOrigin) -> Option<&str> {
    match &origin.origin {
        DataOriginType::DeviceId(id) => Some(id.as_str()),
        DataOriginType::DataOrigin(parent) => extract_collector_id(parent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_cas() -> (TempDir, FsCas) {
        let dir = TempDir::new().unwrap();
        let cas = FsCas::new(dir.path());
        (dir, cas)
    }

    fn ts(secs: i64) -> Option<pbjson_types::Timestamp> {
        Some(pbjson_types::Timestamp {
            seconds: secs,
            nanos: 0,
        })
    }

    #[test]
    fn roundtrip_screen() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::ScreenFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            timestamp: ts(1000),
            width: 1920,
            height: 1080,
            image_bytes: vec![0xFF, 0xD8, 0xFF],
            mime_type: "image/png".to_string(),
            t_device: ts(1000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Screenframe(
                frame.clone(),
            )),
        };
        let row = from_lifelog_data("c1", "screen", &data, &cas).unwrap();
        assert_eq!(row.modality, "Screen");
        assert!(row.blob_hash.is_some());
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Screenframe(f) => {
                assert_eq!(f.width, 1920);
                assert_eq!(f.height, 1080);
                assert_eq!(f.image_bytes, vec![0xFF, 0xD8, 0xFF]);
                assert_eq!(f.mime_type, "image/png");
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_browser() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::BrowserFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            timestamp: ts(2000),
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            visit_count: 3,
            t_device: ts(2000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Browserframe(frame)),
        };
        let row = from_lifelog_data("c1", "browser", &data, &cas).unwrap();
        assert_eq!(row.modality, "Browser");
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Browserframe(f) => {
                assert_eq!(f.url, "https://example.com");
                assert_eq!(f.title, "Example");
                assert_eq!(f.visit_count, 3);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_audio() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::AudioFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440002".to_string(),
            timestamp: ts(3000),
            audio_bytes: vec![0x00, 0x01, 0x02],
            codec: "opus".to_string(),
            sample_rate: 48000,
            channels: 2,
            duration_secs: 5.5,
            t_device: ts(3000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Audioframe(frame)),
        };
        let row = from_lifelog_data("c1", "audio", &data, &cas).unwrap();
        assert_eq!(row.modality, "Audio");
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Audioframe(f) => {
                assert_eq!(f.codec, "opus");
                assert_eq!(f.sample_rate, 48000);
                assert_eq!(f.channels, 2);
                assert_eq!(f.audio_bytes, vec![0x00, 0x01, 0x02]);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_clipboard() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::ClipboardFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440003".to_string(),
            timestamp: ts(4000),
            text: "hello clipboard".to_string(),
            binary_data: vec![],
            mime_type: "text/plain".to_string(),
            t_device: ts(4000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Clipboardframe(frame)),
        };
        let row = from_lifelog_data("c1", "clipboard", &data, &cas).unwrap();
        assert_eq!(row.modality, "Clipboard");
        assert!(row.blob_hash.is_none());
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Clipboardframe(f) => {
                assert_eq!(f.text, "hello clipboard");
                assert_eq!(f.mime_type, "text/plain");
                assert!(f.binary_data.is_empty());
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_shell_history() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::ShellHistoryFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440004".to_string(),
            timestamp: ts(5000),
            command: "ls -la".to_string(),
            working_dir: "/home/user".to_string(),
            exit_code: 0,
            t_device: ts(5000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Shellhistoryframe(
                frame,
            )),
        };
        let row = from_lifelog_data("c1", "shell", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Shellhistoryframe(f) => {
                assert_eq!(f.command, "ls -la");
                assert_eq!(f.working_dir, "/home/user");
                assert_eq!(f.exit_code, 0);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_keystroke() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::KeystrokeFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440005".to_string(),
            timestamp: ts(6000),
            text: "hello world".to_string(),
            application: "firefox".to_string(),
            window_title: "GitHub".to_string(),
            t_device: ts(6000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Keystrokeframe(frame)),
        };
        let row = from_lifelog_data("c1", "keystrokes", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Keystrokeframe(f) => {
                assert_eq!(f.text, "hello world");
                assert_eq!(f.application, "firefox");
                assert_eq!(f.window_title, "GitHub");
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_mouse() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::MouseFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440006".to_string(),
            timestamp: ts(7000),
            x: 500.0,
            y: 300.0,
            button: 1,
            pressed: true,
            t_device: ts(7000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Mouseframe(frame)),
        };
        let row = from_lifelog_data("c1", "mouse", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Mouseframe(f) => {
                assert_eq!(f.x, 500.0);
                assert_eq!(f.y, 300.0);
                assert_eq!(f.button, 1);
                assert!(f.pressed);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_window_activity() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::WindowActivityFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440007".to_string(),
            timestamp: ts(8000),
            application: "code".to_string(),
            window_title: "frames.rs".to_string(),
            focused: true,
            duration_secs: 120.5,
            t_device: ts(8000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Windowactivityframe(
                frame,
            )),
        };
        let row = from_lifelog_data("c1", "windowactivity", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Windowactivityframe(f) => {
                assert_eq!(f.application, "code");
                assert_eq!(f.window_title, "frames.rs");
                assert!(f.focused);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_process() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::ProcessFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440008".to_string(),
            timestamp: ts(9000),
            processes: vec![lifelog_types::ProcessInfo {
                pid: 1234,
                ppid: 1,
                name: "cargo".to_string(),
                exe: "/usr/bin/cargo".to_string(),
                cmdline: "cargo test".to_string(),
                status: "running".to_string(),
                cpu_usage: 45.2,
                memory_usage: 1024000,
                threads: 8,
                user: "matth".to_string(),
                start_time: 1000.0,
            }],
            t_device: ts(9000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Processframe(frame)),
        };
        let row = from_lifelog_data("c1", "processes", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Processframe(f) => {
                assert_eq!(f.processes.len(), 1);
                assert_eq!(f.processes[0].pid, 1234);
                assert_eq!(f.processes[0].name, "cargo");
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_camera() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::CameraFrame {
            uuid: "550e8400-e29b-41d4-a716-446655440009".to_string(),
            timestamp: ts(10000),
            width: 640,
            height: 480,
            image_bytes: vec![0x89, 0x50, 0x4E],
            mime_type: "image/jpeg".to_string(),
            device: "/dev/video0".to_string(),
            t_device: ts(10000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Cameraframe(frame)),
        };
        let row = from_lifelog_data("c1", "camera", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Cameraframe(f) => {
                assert_eq!(f.width, 640);
                assert_eq!(f.height, 480);
                assert_eq!(f.device, "/dev/video0");
                assert_eq!(f.image_bytes, vec![0x89, 0x50, 0x4E]);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_weather() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::WeatherFrame {
            uuid: "550e8400-e29b-41d4-a716-44665544000a".to_string(),
            timestamp: ts(11000),
            temperature: 22.5,
            humidity: 45.0,
            pressure: 1013.25,
            conditions: "Sunny".to_string(),
            t_device: ts(11000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Weatherframe(frame)),
        };
        let row = from_lifelog_data("c1", "weather", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Weatherframe(f) => {
                assert_eq!(f.temperature, 22.5);
                assert_eq!(f.conditions, "Sunny");
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_hyprland() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::HyprlandFrame {
            uuid: "550e8400-e29b-41d4-a716-44665544000b".to_string(),
            timestamp: ts(12000),
            monitors: vec![lifelog_types::HyprMonitor {
                id: 0,
                name: "DP-1".to_string(),
                width: 2560,
                height: 1440,
                ..Default::default()
            }],
            workspaces: vec![lifelog_types::HyprWorkspace {
                id: 1,
                name: "1".to_string(),
                windows: 3,
                ..Default::default()
            }],
            active_workspace: Some(lifelog_types::HyprWorkspace {
                id: 1,
                name: "1".to_string(),
                ..Default::default()
            }),
            clients: vec![],
            devices: vec![],
            cursor: Some(lifelog_types::HyprCursor { x: 100.0, y: 200.0 }),
            t_device: ts(12000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Hyprlandframe(frame)),
        };
        let row = from_lifelog_data("c1", "hyprland", &data, &cas).unwrap();
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Hyprlandframe(f) => {
                assert_eq!(f.monitors.len(), 1);
                assert_eq!(f.monitors[0].name, "DP-1");
                assert_eq!(f.monitors[0].width, 2560);
                assert_eq!(f.workspaces.len(), 1);
                assert!(f.cursor.is_some());
                assert_eq!(f.cursor.unwrap().x, 100.0);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_ocr() {
        let (_dir, cas) = test_cas();
        let source_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let frame = lifelog_types::OcrFrame {
            uuid: "550e8400-e29b-41d4-a716-44665544000c".to_string(),
            timestamp: ts(13000),
            text: "extracted text from screen".to_string(),
            t_device: ts(13000),
            ..Default::default()
        };
        let row = from_ocr("c1", "ocr", &frame, Some(source_id));
        assert_eq!(row.source_frame_id, Some(source_id));
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Ocrframe(f) => {
                assert_eq!(f.text, "extracted text from screen");
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_transcription() {
        let (_dir, cas) = test_cas();
        let source_uuid = "550e8400-e29b-41d4-a716-446655440002";
        let frame = lifelog_types::TranscriptionFrame {
            uuid: "550e8400-e29b-41d4-a716-44665544000d".to_string(),
            timestamp: ts(14000),
            text: "hello world from audio".to_string(),
            source_uuid: source_uuid.to_string(),
            model: "whisper-large-v3".to_string(),
            confidence: 0.95,
            t_device: ts(14000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Transcriptionframe(
                frame,
            )),
        };
        let row = from_lifelog_data("c1", "transcription", &data, &cas).unwrap();
        assert!(row.source_frame_id.is_some());
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Transcriptionframe(f) => {
                assert_eq!(f.text, "hello world from audio");
                assert_eq!(f.model, "whisper-large-v3");
                assert!((f.confidence - 0.95).abs() < 0.01);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn roundtrip_embedding() {
        let (_dir, cas) = test_cas();
        let frame = lifelog_types::EmbeddingFrame {
            uuid: "550e8400-e29b-41d4-a716-44665544000e".to_string(),
            timestamp: ts(15000),
            source_uuid: "550e8400-e29b-41d4-a716-44665544000c".to_string(),
            model: "clip-vit-b32".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4],
            dimensions: 4,
            t_device: ts(15000),
            ..Default::default()
        };
        let data = lifelog_types::LifelogData {
            payload: Some(lifelog_types::lifelog_data::Payload::Embeddingframe(frame)),
        };
        let row = from_lifelog_data("c1", "embedding", &data, &cas).unwrap();
        assert_eq!(row.modality, "VectorEmbedding");
        let rt = to_lifelog_data(&row, &cas).unwrap();
        match rt.payload.unwrap() {
            lifelog_types::lifelog_data::Payload::Embeddingframe(f) => {
                assert_eq!(f.model, "clip-vit-b32");
                assert_eq!(f.dimensions, 4);
                assert_eq!(f.vector.len(), 4);
            }
            _ => panic!("wrong payload type"),
        }
    }

    #[test]
    fn no_payload_errors() {
        let (_dir, cas) = test_cas();
        let data = lifelog_types::LifelogData { payload: None };
        assert!(from_lifelog_data("c1", "x", &data, &cas).is_err());
    }

    #[test]
    fn insert_sql_is_valid() {
        let sql = FrameRow::insert_sql();
        assert!(sql.contains("INSERT INTO frames"));
        assert!(sql.contains("ON CONFLICT (id) DO NOTHING"));
        assert!(sql.contains("$16"));
    }
}
