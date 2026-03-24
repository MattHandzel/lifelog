use chrono::{DateTime, Utc};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::FrameRow;
use utils::cas::FsCas;

fn pb_to_dt(ts: Option<pbjson_types::Timestamp>) -> DateTime<Utc> {
    let ts = ts.unwrap_or_default();
    DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_else(|| {
        DateTime::<Utc>::from_naive_utc_and_offset(chrono::NaiveDateTime::MIN, Utc)
    })
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
        modality: "Keystroke".to_string(),
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
