use clap::{Arg, Command};
use lifelog::setup;
use rusqlite::*;

use lifelog::config::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::join;

enum DataSourceKind {
    Hyprland {
        path_to_db: PathBuf,
    },
    Screen {
        path_to_db: PathBuf,
        path_to_dir: PathBuf,
    },
    Processes {
        path_to_db: PathBuf,
    },
    Microphone {
        path_to_db: PathBuf,
        path_to_dir: PathBuf,
    },
}

enum ConfigKind {
    Hyprland(HyprlandConfig),
    Screen(ScreenConfig),
    Processes(ProcessesConfig),
    Microphone(MicrophoneConfig),
}

struct DataSource {
    kind: DataSourceKind,
    conn: Connection,
}

fn data_source_kind_to_table_names(kind: DataSourceKind) -> Vec<String> {
    match kind {
        DataSourceKind::Hyprland { .. } => vec![
            "clients",
            "devices",
            "cursor_positions",
            "activeworkspace",
            "workspaces",
            "monitors",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        DataSourceKind::Screen { .. } => vec!["screen".to_string()],
        DataSourceKind::Processes { .. } => vec!["processes".to_string()],
        DataSourceKind::Microphone { .. } => vec!["microphone".to_string()],
    }
}

//fn identify_data_source_kind(directory: PathBuf) -> DataSourceKind {
//    for entry in std::fs::read_dir(directory.clone()).expect("Directory not found") {
//        let entry = entry.expect("Failed to read entry");
//        let path = entry.path();
//        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("db") {
//            return DataSourceKind::Database {
//                database_path: path.to_str().unwrap().to_string(),
//            };
//        }
//    }
//    DataSourceKind::Blob {
//        directory: directory.to_str().unwrap().to_string(),
//    }
//}

//fn open_data_source(data_source: DataSource) {
//    match data_source.kind {
//        DataSourceKind::Blob { directory } => {
//            println!("Opening blob data source: {}", directory);
//        }
//        DataSourceKind::Database { database_path } => {
//            println!("Opening database data source: {}", database_path);
//        }
//    }
//}

fn execute_query_on_table(conn: &Connection, table_name: &str, query: &str) -> Result<()> {
    // Prepare the full SQL statement by appending the user's query to a SELECT statement
    let sql = format!("SELECT * FROM {} WHERE {}", table_name, query);

    // Prepare and execute the SQL statement
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        // This closure processes each row of the result
        // For simplicity, we'll just print the row as a debug string
        Ok(format!("{:?}", row))
    })?;

    // Iterate over the rows and print the results
    for row in rows {
        println!("{}", row?);
    }

    Ok(())
}

fn main() {
    let config = load_config();

    // create a hashmap between name and the database getter
    // Ex: "hyprland" : setup_hyprland_db

    let matches = Command::new("lifelog-server")
        .version("0.1.0")
        .author("Matthew Handzel <handzelmatthew@gmail.com>")
        .about("Lifelog Server")
        .arg(Arg::new("query").short('q').long("query").takes_value(true))
        .get_matches();

    // Get the value of the "config" argument

    let data_sources: Vec<_> = [
        ConfigKind::Hyprland(config.hyprland),
        ConfigKind::Screen(config.screen),
        ConfigKind::Processes(config.processes),
    ]
    .iter()
    //.filter_map(|(config)| match config {
    //    ConfigKind::Hyprland(.., enabled) => if enabled { Some(ConfigKind::Hyprland(config) } else {None()}),
    //}) // this is bs 😭
    .map(|_config| -> DataSource {
        match _config {
            ConfigKind::Hyprland(HyprlandConfig {
                enabled,
                interval,
                output_dir,
                log_clients,
                log_activewindow,
                log_workspace,
                log_active_monitor,
                log_devices,
            }) => DataSource {
                kind: DataSourceKind::Hyprland {
                    path_to_db: output_dir.join("hyprland.db"),
                },
                conn: setup::setup_hyprland_db(output_dir).expect("Failed to setup hyprland db"),
            },
            ConfigKind::Screen(ScreenConfig {
                enabled,
                interval,
                output_dir,
                program,
                timestamp_format,
            }) => DataSource {
                kind: DataSourceKind::Screen {
                    path_to_dir: output_dir.clone(),
                    path_to_db: output_dir.join("screen.db"),
                },
                conn: setup::setup_screen_db(output_dir).expect("Failed to setup screen db"),
            },
            ConfigKind::Processes(ProcessesConfig {
                enabled,
                interval,
                output_dir,
            }) => DataSource {
                kind: DataSourceKind::Processes {
                    path_to_db: output_dir.join("processes.db"),
                },
                conn: setup::setup_process_db(output_dir).expect("Failed to setup processes db"),
            },
            ConfigKind::Microphone(MicrophoneConfig { output_dir, .. }) => DataSource {
                kind: DataSourceKind::Microphone {
                    path_to_dir: output_dir.clone(),
                    path_to_db: output_dir.join("microphone.db"),
                },
                conn: setup::setup_microphone_db(output_dir)
                    .expect("Failed to setup microphone db"),
            },
        }
    })
    .collect();

    println!("Data sources opened");

    if let Some(query) = matches.value_of("query") {
        println!("Query: {}", query);

        for data_source in data_sources {
            let table_names = data_source_kind_to_table_names(data_source.kind);

            for table_name in table_names {
                println!("Executing query on table: {}", table_name);
                if let Err(e) = execute_query_on_table(&data_source.conn, &table_name, query) {
                    eprintln!("Error executing query on table {}: {}", table_name, e);
                }
            }
        }
    }
}
