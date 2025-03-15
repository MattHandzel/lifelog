// src/modules/hyprland.rs
use serde_json::Value;
use std::process::Command;
use std::collections::HashMap;

fn parse_hyprctl_output(output: &str) -> HyprlandState {
    let mut data: HashMap<&str, &str> = HashMap::new();
    
    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            data.insert(key.trim(), value.trim());
        }
    }

    HyprlandState {
        active_window: data.get("title").unwrap_or(&"").to_string(),
        workspace: data.get("workspace").and_then(|s| s.split_whitespace().next()).unwrap_or("0").parse().unwrap_or(0),
        active_monitor: data.get("monitor").unwrap_or(&"").to_string(),
    }
}

pub struct HyprlandState {
    pub active_window: String,
    pub workspace: u32,
    pub active_monitor: String,
}

pub fn get_hyprland_state() -> HyprlandState {
    let output = Command::new("hyprctl")
        .arg("activewindow")
        .arg("-j")
        .output()
        .expect("Failed to execute hyprctl");
    
    let v: Value = serde_json::from_slice(&output.stdout).unwrap();
    
    HyprlandState {
        active_window: v["title"].as_str().unwrap_or_default().to_string(),
        workspace: v["workspace"]["id"].as_u64().unwrap_or(0) as u32,
        active_monitor: v["monitor"].as_str().unwrap_or_default().to_string(),
    }
}
