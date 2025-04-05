//use crate::config::InputLoggerConfig;
//use crate::prelude::*;
//use crate::setup;
//use anyhow::Result;
//use chrono::Local;
//use rusqlite::{params, Connection};
//use std::sync::Arc;
//use tokio::sync::mpsc;
//use tokio::time::{sleep, Duration};
//use wayland_client::{
//    protocol::{wl_compositor, wl_keyboard, wl_pointer, wl_seat, wl_surface},
//    Connection as WlConnection, Dispatch, EventQueue, GlobalManager, Interface, Main, Proxy,
//    QueueHandle, WEnum,
//};
//use wayland_protocols::wlr::unstable::layer_shell::v1::client::{
//    zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
//};
//use wayland_protocols::xdg::shell::client::xdg_positioner::Anchor;
//
//struct AppData {
//    config: Arc<InputLoggerConfig>,
//    tx: mpsc::Sender<InputEvent>,
//    conn: WlConnection,
//    seat: Option<Main<wl_seat::WlSeat>>,
//    keyboard: Option<Main<wl_keyboard::WlKeyboard>>,
//    pointer: Option<Main<wl_pointer::WlPointer>>,
//    layer_shell: Option<Main<ZwlrLayerShellV1>>,
//    layer_surface: Option<Main<ZwlrLayerSurfaceV1>>,
//    // For tracking mouse position deltas
//    last_mouse_position: (f64, f64),
//}
//
//pub async fn start_logger(config: &InputLoggerConfig) -> Result<()> {
//    let config = config.clone();
//    let conn = setup::setup_input_logger_db(&config.output_dir)?;
//    let (tx, mut rx) = mpsc::channel(256);
//
//    // Connect to Wayland
//    let conn_wayland = WlConnection::connect_to_env()?;
//    let mut event_queue = conn_wayland.new_event_queue();
//    let qh = event_queue.handle();
//
//    let global_manager = GlobalManager::new(&conn_wayland);
//
//    // First roundtrip to get registry events
//    event_queue.roundtrip(&mut ())?;
//
//    // Create compositor and surface
//    let compositor = global_manager.instantiate_exact::<wl_compositor::WlCompositor>(4)?;
//    let surface = compositor.create_surface(&qh);
//
//    // Setup layer shell
//    let layer_shell = global_manager.instantiate_exact::<ZwlrLayerShellV1>(1)?;
//    let layer_surface = layer_shell.get_layer_surface(
//        &surface,
//        None,
//        zwlr_layer_shell_v1::Layer::Overlay,
//        "input-logger",
//        &qh,
//    );
//
//    // Configure layer surface
//    layer_surface.set_size(1, 1);
//    layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::TOP);
//    layer_surface.set_exclusive_zone(-1);
//    surface.commit();
//
//    let mut app_data = AppData {
//        config: Arc::new(config),
//        tx,
//        conn: conn_wayland.clone(),
//        seat: None,
//        keyboard: None,
//        pointer: None,
//        layer_shell: Some(layer_shell),
//        layer_surface: Some(layer_surface),
//        last_mouse_position: (0.0, 0.0),
//    };
//
//    global_manager.instantiate_auto::<wl_seat::WlSeat>(&qh, 1..=7, ())?;
//
//    // Main event loop
//    loop {
//        event_queue.dispatch_pending()?;
//        event_queue.flush()?;
//
//        while let Ok(event) = rx.try_recv() {
//            match event {
//                InputEvent::KeyEvent {
//                    key,
//                    pressed,
//                    timestamp,
//                } => {
//                    log_key_event(&conn, pressed, &key, timestamp)?;
//                }
//                InputEvent::MouseEvent {
//                    button,
//                    pressed,
//                    timestamp,
//                } => {
//                    log_mouse_button(&conn, pressed, &button, timestamp)?;
//                }
//                InputEvent::MouseMove { x, y, timestamp } => {
//                    log_mouse_move(&conn, x, y, timestamp)?;
//                }
//                InputEvent::MouseWheel {
//                    delta_x,
//                    delta_y,
//                    timestamp,
//                } => {
//                    log_mouse_wheel(&conn, delta_x, delta_y, timestamp)?;
//                }
//            }
//        }
//
//        sleep(Duration::from_secs_f64(0.001)).await;
//    }
//}
//
//impl Dispatch<wl_seat::WlSeat, ()> for AppData {
//    fn event(
//        state: &mut Self,
//        seat: &wl_seat::WlSeat,
//        event: wl_seat::Event,
//        _: &(),
//        _: &WlConnection,
//        qh: &QueueHandle<Self>,
//    ) {
//        if let wl_seat::Event::Capabilities { capabilities } = event {
//            if let WEnum::Value(caps) = capabilities {
//                if caps.contains(wl_seat::Capability::Keyboard) {
//                    let keyboard = seat.get_keyboard(qh, ());
//                    state.keyboard = Some(keyboard);
//                }
//                if caps.contains(wl_seat::Capability::Pointer) {
//                    let pointer = seat.get_pointer(qh, ());
//                    state.pointer = Some(pointer);
//                }
//            }
//        }
//    }
//}
//
//impl Dispatch<wl_keyboard::WlKeyboard, ()> for AppData {
//    fn event(
//        state: &mut Self,
//        _: &wl_keyboard::WlKeyboard,
//        event: wl_keyboard::Event,
//        _: &(),
//        _: &WlConnection,
//        _: &QueueHandle<Self>,
//    ) {
//        if state.config.log_keyboard {
//            match event {
//                wl_keyboard::Event::Key {
//                    key,
//                    state: key_state,
//                    ..
//                } => {
//                    let timestamp = Local::now().timestamp() as f64
//                        + Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
//                    let pressed = key_state == WEnum::Value(wl_keyboard::KeyState::Pressed);
//                    let key_str = format!("{}", key);
//
//                    let tx = state.tx.clone();
//                    tokio::spawn(async move {
//                        if let Err(e) = tx
//                            .send(InputEvent::KeyEvent {
//                                key: key_str,
//                                pressed,
//                                timestamp,
//                            })
//                            .await
//                        {
//                            log::error!("Failed to send key event: {}", e);
//                        }
//                    });
//                }
//                _ => (),
//            }
//        }
//    }
//}
//
//impl Dispatch<wl_pointer::WlPointer, ()> for AppData {
//    fn event(
//        state: &mut Self,
//        _: &wl_pointer::WlPointer,
//        event: wl_pointer::Event,
//        _: &(),
//        _: &WlConnection,
//        _: &QueueHandle<Self>,
//    ) {
//        let timestamp = Local::now().timestamp() as f64
//            + Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
//
//        match event {
//            wl_pointer::Event::Button {
//                button,
//                state: btn_state,
//                ..
//            } if state.config.log_mouse_buttons => {
//                let pressed = btn_state == WEnum::Value(wl_pointer::ButtonState::Pressed);
//                let button_str = format!("{}", button);
//
//                let tx = state.tx.clone();
//                tokio::spawn(async move {
//                    if let Err(e) = tx
//                        .send(InputEvent::MouseEvent {
//                            button: button_str,
//                            pressed,
//                            timestamp,
//                        })
//                        .await
//                    {
//                        log::error!("Failed to send mouse event: {}", e);
//                    }
//                });
//            }
//            wl_pointer::Event::Motion {
//                surface_x,
//                surface_y,
//                ..
//            } if state.config.log_mouse_movement => {
//                // Calculate delta from last position
//                let dx = surface_x - state.last_mouse_position.0;
//                let dy = surface_y - state.last_mouse_position.1;
//                state.last_mouse_position = (surface_x, surface_y);
//
//                let tx = state.tx.clone();
//                tokio::spawn(async move {
//                    if let Err(e) = tx
//                        .send(InputEvent::MouseMove {
//                            x: dx,
//                            y: dy,
//                            timestamp,
//                        })
//                        .await
//                    {
//                        log::error!("Failed to send mouse move: {}", e);
//                    }
//                });
//            }
//            _ => (),
//        }
//    }
//}
//
//// Keep existing InputEvent enum and logging functions
//#[derive(Debug)]
//enum InputEvent {
//    KeyEvent {
//        key: String,
//        pressed: bool,
//        timestamp: f64,
//    },
//    MouseEvent {
//        button: String,
//        pressed: bool,
//        timestamp: f64,
//    },
//    MouseMove {
//        x: f64,
//        y: f64,
//        timestamp: f64,
//    },
//    MouseWheel {
//        delta_x: i64,
//        delta_y: i64,
//        timestamp: f64,
//    },
//}
//
//fn log_key_event(conn: &Connection, pressed: bool, key: &str, timestamp: f64) -> Result<()> {
//    println!("Key: {:?} {:?}", key, pressed);
//    conn.execute(
//        "INSERT INTO key_events VALUES (?1, ?2, ?3)",
//        params![timestamp, if pressed { "press" } else { "release" }, key],
//    )?;
//    Ok(())
//}
//
//fn log_mouse_button(conn: &Connection, pressed: bool, button: &str, timestamp: f64) -> Result<()> {
//    println!("Mouse: {:?} {:?}", button, pressed);
//    conn.execute(
//        "INSERT INTO mouse_buttons VALUES (?1, ?2, ?3)",
//        params![timestamp, if pressed { "press" } else { "release" }, button],
//    )?;
//    Ok(())
//}
//
//fn log_mouse_move(conn: &Connection, x: f64, y: f64, timestamp: f64) -> Result<()> {
//    conn.execute(
//        "INSERT INTO mouse_movements VALUES (?1, ?2, ?3)",
//        params![timestamp, x, y],
//    )?;
//    Ok(())
//}
//
//fn log_mouse_wheel(conn: &Connection, delta_x: i64, delta_y: i64, timestamp: f64) -> Result<()> {
//    conn.execute(
//        "INSERT INTO mouse_wheel VALUES (?1, ?2, ?3)",
//        params![timestamp, delta_x, delta_y],
//    )?;
//    Ok(())
//}
