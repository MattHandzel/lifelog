use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

use super::FrameRow;
use utils::cas::FsCas;

fn dt_to_pb(dt: DateTime<Utc>) -> pbjson_types::Timestamp {
    pbjson_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
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
        "Keystroke" | "Keystrokes" => Payload::Keystrokeframe(lifelog_types::KeystrokeFrame {
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
