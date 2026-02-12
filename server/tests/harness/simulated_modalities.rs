#![allow(clippy::expect_used, dead_code)]

use chrono::{DateTime, Utc};
use lifelog_types::{
    to_pb_ts, AudioFrame, BrowserFrame, CameraFrame, ClipboardFrame, DataModality, HyprClient,
    HyprCursor, HyprDevice, HyprMonitor, HyprWorkspace, HyprlandFrame, KeystrokeFrame, LifelogData,
    MouseFrame, ProcessFrame, ProcessInfo, RecordType, ScreenFrame, ShellHistoryFrame,
    WeatherFrame, WindowActivityFrame,
};
use prost::Message;

#[derive(Debug, Clone)]
pub struct SimulatedModalityChunk {
    pub stream_id: &'static str,
    pub modality: DataModality,
    pub uuid: String,
    pub chunk: lifelog_types::Chunk,
}

pub fn simulated_all_modality_chunks(
    collector_id: &str,
    session_id: u64,
    base: DateTime<Utc>,
) -> Vec<SimulatedModalityChunk> {
    let mut out = Vec::new();

    let mut push =
        |stream_id: &'static str, modality: DataModality, uuid: String, payload: Vec<u8>| {
            out.push(SimulatedModalityChunk {
                stream_id,
                modality,
                uuid,
                chunk: lifelog_types::Chunk {
                    stream: Some(lifelog_types::StreamIdentity {
                        collector_id: collector_id.to_string(),
                        stream_id: stream_id.to_string(),
                        session_id,
                    }),
                    offset: 0,
                    hash: utils::cas::sha256_hex(&payload),
                    data: payload,
                },
            });
        };

    let t0 = to_pb_ts(base);
    let t1 = to_pb_ts(base + chrono::Duration::seconds(1));

    // Screen
    let screen = ScreenFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        width: 1920,
        height: 1080,
        image_bytes: vec![137, 80, 78, 71, 0, 1, 2, 3],
        mime_type: "image/png".to_string(),
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    screen.encode(&mut buf).expect("encode screen");
    push("screen", DataModality::Screen, screen.uuid.clone(), buf);

    // Browser
    let browser = BrowserFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        url: "https://example.com/all-modalities".to_string(),
        title: "All Modalities".to_string(),
        visit_count: 1,
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    browser.encode(&mut buf).expect("encode browser");
    push("browser", DataModality::Browser, browser.uuid.clone(), buf);

    // Audio
    let audio = AudioFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        audio_bytes: vec![1; 128],
        codec: "wav".to_string(),
        sample_rate: 48_000,
        channels: 1,
        duration_secs: 1.0,
        t_device: t0,
        t_canonical: t0,
        t_end: t1,
        record_type: RecordType::Interval as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    audio.encode(&mut buf).expect("encode audio");
    push("audio", DataModality::Audio, audio.uuid.clone(), buf);

    // Keystrokes
    let keystroke = KeystrokeFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        text: "KeyA".to_string(),
        application: "test-app".to_string(),
        window_title: "test-window".to_string(),
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    keystroke.encode(&mut buf).expect("encode keystroke");
    push(
        "keystrokes",
        DataModality::Keystrokes,
        keystroke.uuid.clone(),
        buf,
    );

    // Clipboard
    let clipboard = ClipboardFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        text: "clipboard text".to_string(),
        binary_data: vec![1, 2, 3, 4],
        mime_type: "application/octet-stream".to_string(),
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    clipboard.encode(&mut buf).expect("encode clipboard");
    push(
        "clipboard",
        DataModality::Clipboard,
        clipboard.uuid.clone(),
        buf,
    );

    // Shell history
    let shell = ShellHistoryFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        command: "echo all-modalities".to_string(),
        working_dir: "/tmp".to_string(),
        exit_code: 0,
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    shell.encode(&mut buf).expect("encode shell");
    push(
        "shell_history",
        DataModality::ShellHistory,
        shell.uuid.clone(),
        buf,
    );

    // Window activity
    let window = WindowActivityFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        application: "firefox".to_string(),
        window_title: "All Modalities - Mozilla Firefox".to_string(),
        focused: true,
        duration_secs: 1.0,
        t_device: t0,
        t_canonical: t0,
        t_end: t1,
        record_type: RecordType::Interval as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    window.encode(&mut buf).expect("encode window activity");
    push(
        "window_activity",
        DataModality::WindowActivity,
        window.uuid.clone(),
        buf,
    );

    // Mouse
    let mouse = MouseFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        x: 100.0,
        y: 200.0,
        button: lifelog_types::mouse_frame::MouseButton::Left as i32,
        pressed: true,
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    mouse.encode(&mut buf).expect("encode mouse");
    push("mouse", DataModality::Mouse, mouse.uuid.clone(), buf);

    // Processes
    let proc = ProcessFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        processes: vec![ProcessInfo {
            pid: 1234,
            ppid: 1,
            name: "test-process".to_string(),
            exe: "/usr/bin/test-process".to_string(),
            cmdline: "test-process --arg".to_string(),
            status: "Running".to_string(),
            cpu_usage: 0.1,
            memory_usage: 1024,
            threads: 4,
            user: "tester".to_string(),
            start_time: base.timestamp() as f64,
        }],
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    proc.encode(&mut buf).expect("encode process");
    push("processes", DataModality::Processes, proc.uuid.clone(), buf);

    // Camera
    let camera = CameraFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        width: 640,
        height: 480,
        image_bytes: vec![255, 216, 255, 224, 0, 16],
        mime_type: "image/jpeg".to_string(),
        device: "/dev/video0".to_string(),
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    camera.encode(&mut buf).expect("encode camera");
    push("camera", DataModality::Camera, camera.uuid.clone(), buf);

    // Weather
    let weather = WeatherFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        temperature: 21.5,
        humidity: 50.0,
        pressure: 1012.0,
        conditions: "Clear".to_string(),
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    weather.encode(&mut buf).expect("encode weather");
    push("weather", DataModality::Weather, weather.uuid.clone(), buf);

    // Hyprland
    let hypr = HyprlandFrame {
        uuid: lifelog_core::Uuid::new_v4().to_string(),
        timestamp: t0,
        monitors: vec![HyprMonitor {
            id: 0,
            name: "HDMI-A-1".to_string(),
            description: "Test monitor".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: 60.0,
            x: 0,
            y: 0,
            workspace_id: 1,
            workspace_name: "1".to_string(),
            scale: 1.0,
            focused: true,
        }],
        workspaces: vec![HyprWorkspace {
            id: 1,
            name: "1".to_string(),
            monitor: "HDMI-A-1".to_string(),
            monitor_id: 0,
            windows: 1,
            fullscreen: false,
            last_window: "0x1".to_string(),
            last_window_title: "all modalities".to_string(),
        }],
        active_workspace: Some(HyprWorkspace {
            id: 1,
            name: "1".to_string(),
            monitor: "HDMI-A-1".to_string(),
            monitor_id: 0,
            windows: 1,
            fullscreen: false,
            last_window: "0x1".to_string(),
            last_window_title: "all modalities".to_string(),
        }),
        clients: vec![HyprClient {
            address: "0x1".to_string(),
            x: 0,
            y: 0,
            width: 100,
            height: 100,
            workspace_id: 1,
            workspace_name: "1".to_string(),
            floating: false,
            fullscreen: "none".to_string(),
            monitor: 0,
            title: "all modalities".to_string(),
            class: "test".to_string(),
            pid: 1234,
            pinned: false,
            mapped: true,
        }],
        devices: vec![HyprDevice {
            r#type: "keyboard".to_string(),
            name: "test-kbd".to_string(),
            address: "kbd-1".to_string(),
        }],
        cursor: Some(HyprCursor { x: 10.0, y: 20.0 }),
        t_device: t0,
        t_canonical: t0,
        t_end: t0,
        record_type: RecordType::Point as i32,
        ..Default::default()
    };
    let mut buf = Vec::new();
    hypr.encode(&mut buf).expect("encode hyprland");
    push("hyprland", DataModality::Hyprland, hypr.uuid.clone(), buf);

    out
}

pub fn payload_matches_modality(modality: DataModality, data: &LifelogData) -> bool {
    matches!(
        (&data.payload, modality),
        (
            Some(lifelog_types::lifelog_data::Payload::Screenframe(_)),
            DataModality::Screen
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Browserframe(_)),
            DataModality::Browser
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Audioframe(_)),
            DataModality::Audio
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Keystrokeframe(_)),
            DataModality::Keystrokes,
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Clipboardframe(_)),
            DataModality::Clipboard,
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Shellhistoryframe(_)),
            DataModality::ShellHistory,
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Windowactivityframe(_)),
            DataModality::WindowActivity,
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Mouseframe(_)),
            DataModality::Mouse
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Processframe(_)),
            DataModality::Processes,
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Cameraframe(_)),
            DataModality::Camera
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Weatherframe(_)),
            DataModality::Weather
        ) | (
            Some(lifelog_types::lifelog_data::Payload::Hyprlandframe(_)),
            DataModality::Hyprland,
        )
    )
}
