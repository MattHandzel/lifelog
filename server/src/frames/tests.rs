use super::*;
use tempfile::TempDir;
use utils::cas::FsCas;
use uuid::Uuid;

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
