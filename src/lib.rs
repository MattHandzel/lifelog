pub mod config;
pub mod setup;
pub mod modules {
    pub mod camera;
    pub mod evdev_input_logger;
    pub mod hyprland;
    pub mod input_logger;
    pub mod microphone;
    pub mod mouse;
    pub mod wayland_input_logger;

    //pub mod logger;
    pub mod processes;
    pub mod screen;
    pub mod weather;
}

//pub mod embed;
pub mod prelude;
pub mod utils;
