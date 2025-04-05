use crate::prelude::*;
use anyhow::Result;
use chrono::Local;
use config::InputLoggerConfig;
use rdev::{listen, Event, EventType};
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub async fn start_logger(config: &InputLoggerConfig) -> Result<()> {
    let config = config.clone();
    let conn = setup::setup_input_logger_db(&config.output_dir)?;
    let (tx, mut rx) = mpsc::channel(256);

    // Spawn the input listener thread (blocking task)
    let config_clone = Arc::new(config.clone());
    tokio::task::spawn_blocking(move || {
        let callback = move |event: Event| {
            let timestamp = Local::now().timestamp() as f64
                + Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

            let tx = tx.clone();
            match event.event_type {
                EventType::KeyPress(key) if config_clone.log_keyboard => {
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(InputEvent::KeyEvent {
                                key: format!("{:?}", key),
                                pressed: true,
                                timestamp,
                            })
                            .await
                        {
                            log::error!("Failed to send key event: {}", e);
                        }
                    });
                }
                EventType::KeyRelease(key) if config_clone.log_keyboard => {
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(InputEvent::KeyEvent {
                                key: format!("{:?}", key),
                                pressed: false,
                                timestamp,
                            })
                            .await
                        {
                            log::error!("Failed to send key event: {}", e);
                        }
                    });
                }
                EventType::ButtonPress(button) if config_clone.log_mouse_buttons => {
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(InputEvent::MouseEvent {
                                button: format!("{:?}", button),
                                pressed: true,
                                timestamp,
                            })
                            .await
                        {
                            log::error!("Failed to send mouse event: {}", e);
                        }
                    });
                }
                EventType::ButtonRelease(button) if config_clone.log_mouse_buttons => {
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(InputEvent::MouseEvent {
                                button: format!("{:?}", button),
                                pressed: false,
                                timestamp,
                            })
                            .await
                        {
                            log::error!("Failed to send mouse event: {}", e);
                        }
                    });
                }
                EventType::MouseMove { x, y } if config_clone.log_mouse_movement => {
                    tokio::spawn(async move {
                        if let Err(e) = tx.send(InputEvent::MouseMove { x, y, timestamp }).await {
                            log::error!("Failed to send mouse move: {}", e);
                        }
                    });
                }
                EventType::Wheel { delta_x, delta_y } if config_clone.log_mouse_wheel => {
                    tokio::spawn(async move {
                        if let Err(e) = tx
                            .send(InputEvent::MouseWheel {
                                delta_x,
                                delta_y,
                                timestamp,
                            })
                            .await
                        {
                            log::error!("Failed to send mouse wheel: {}", e);
                        }
                    });
                }
                _ => (),
            }
        };

        if let Err(e) = listen(callback) {
            log::error!("Input listener error: {:?}", e);
        }
    });

    // Main logging loop
    loop {
        // Process any pending events
        while let Ok(event) = rx.try_recv() {
            match event {
                InputEvent::KeyEvent {
                    key,
                    pressed,
                    timestamp,
                } => {
                    log_key_event(&conn, pressed, &key, timestamp)?;
                }
                InputEvent::MouseEvent {
                    button,
                    pressed,
                    timestamp,
                } => {
                    log_mouse_button(&conn, pressed, &button, timestamp)?;
                }
                InputEvent::MouseMove { x, y, timestamp } => {
                    log_mouse_move(&conn, x, y, timestamp)?;
                }
                InputEvent::MouseWheel {
                    delta_x,
                    delta_y,
                    timestamp,
                } => {
                    log_mouse_wheel(&conn, delta_x, delta_y, timestamp)?;
                }
            }
        }

        // Small sleep to prevent busy waiting
        sleep(Duration::from_millis(10)).await;
    }
}

#[derive(Debug)]
enum InputEvent {
    KeyEvent {
        key: String,
        pressed: bool,
        timestamp: f64,
    },
    MouseEvent {
        button: String,
        pressed: bool,
        timestamp: f64,
    },
    MouseMove {
        x: f64,
        y: f64,
        timestamp: f64,
    },
    MouseWheel {
        delta_x: i64,
        delta_y: i64,
        timestamp: f64,
    },
}

fn log_key_event(conn: &Connection, pressed: bool, key: &str, timestamp: f64) -> Result<()> {
    println!("Key: {:?} {:?}", key, pressed);
    conn.execute(
        "INSERT INTO key_events VALUES (?1, ?2, ?3)",
        params![timestamp, if pressed { "press" } else { "release" }, key],
    )?;
    Ok(())
}

fn log_mouse_button(conn: &Connection, pressed: bool, button: &str, timestamp: f64) -> Result<()> {
    println!("Mouse: {:?} {:?}", button, pressed);
    conn.execute(
        "INSERT INTO mouse_buttons VALUES (?1, ?2, ?3)",
        params![timestamp, if pressed { "press" } else { "release" }, button],
    )?;
    Ok(())
}

fn log_mouse_move(conn: &Connection, x: f64, y: f64, timestamp: f64) -> Result<()> {
    conn.execute(
        "INSERT INTO mouse_movements VALUES (?1, ?2, ?3)",
        params![timestamp, x, y],
    )?;
    Ok(())
}

fn log_mouse_wheel(conn: &Connection, delta_x: i64, delta_y: i64, timestamp: f64) -> Result<()> {
    conn.execute(
        "INSERT INTO mouse_wheel VALUES (?1, ?2, ?3)",
        params![timestamp, delta_x, delta_y],
    )?;
    Ok(())
}
