use clap::{Arg, Command};
use lifelog::setup;
use rusqlite::*;
use rusqlite::types::Value;
use lifelog::config::*;
use std::path::{Path, PathBuf};

enum DataSourceKind {
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
    Screen(ScreenConfig),
    Processes(ProcessesConfig),
    Microphone(MicrophoneConfig),
}

struct DataSource {
    kind: DataSourceKind,
    conn: Connection,
}

fn data_source_kind_to_table_names(kind: &DataSourceKind) -> Vec<String> {
    match kind {
        DataSourceKind::Screen { .. } => vec!["screen".to_string()],
        DataSourceKind::Processes { .. } => vec!["processes".to_string()],
        DataSourceKind::Microphone { .. } => vec!["microphone".to_string()],
    }
}

fn main() {
    let config = load_config();

    let matches = Command::new("lifelog-server")
        .version("0.1.0")
        .author("Matthew Handzel <handzelmatthew@gmail.com>")
        .about("Lifelog Server")
        .arg(Arg::new("query").short('q').long("query").takes_value(true))
        .get_matches();

    let data_sources: Vec<_> = [
        ConfigKind::Screen(config.screen),
        ConfigKind::Processes(config.processes),
        ConfigKind::Microphone(config.microphone),
    ]
    .iter()
    .map(|config| match config {
        ConfigKind::Screen(ScreenConfig {
            enabled,
            interval: _,
            output_dir,
            program: _,
            timestamp_format: _,
        }) => {
            if !enabled {
                return None;
            }
            Some(DataSource {
                kind: DataSourceKind::Screen {
                    path_to_dir: output_dir.clone(),
                    path_to_db: output_dir.join("screen.db"),
                },
                conn: setup::setup_screen_db(output_dir).expect("Failed to setup screen db"),
            })
        }
        ConfigKind::Processes(ProcessesConfig {
            enabled,
            interval: _,
            output_dir,
        }) => {
            if !enabled {
                return None;
            }
            Some(DataSource {
                kind: DataSourceKind::Processes {
                    path_to_db: output_dir.join("processes.db"),
                },
                conn: setup::setup_process_db(output_dir).expect("Failed to setup processes db"),
            })
        }
        ConfigKind::Microphone(MicrophoneConfig {
            enabled,
            output_dir,
            ..
        }) => {
            if !enabled {
                return None;
            }
            Some(DataSource {
                kind: DataSourceKind::Microphone {
                    path_to_dir: output_dir.clone(),
                    path_to_db: output_dir.join("microphone.db"),
                },
                conn: Connection::open(output_dir.join("microphone.db"))
                    .expect("Failed to setup microphone db"),
            })
        }
    })
    .filter_map(|x| x)
    .collect();

    // Handle the query if provided
    if let Some(query) = matches.value_of("query") {
        for source in &data_sources {
            let table_names = data_source_kind_to_table_names(&source.kind);
            for table in table_names {
                let mut stmt = source
                    .conn
                    .prepare(&format!("SELECT * FROM {} WHERE {}", table, query))
                    .unwrap();
                
                println!("Results from table {}:", table);
                let names: Vec<_> = stmt.column_names().into_iter().map(|s| s.to_string()).collect();
                let rows = stmt.query_map([], |row| {
                    let mut values = Vec::new();
                    for (i, name) in names.iter().enumerate() {
                        values.push(format!("{}: {:?}", 
                            name,
                            row.get::<_, Value>(i).unwrap_or(Value::Null)
                        ));
                    }
                    Ok(values)
                }).unwrap();

                for row_result in rows {
                    match row_result {
                        Ok(values) => println!("{:?}", values),
                        Err(e) => eprintln!("Error reading row: {}", e),
                    }
                }
            }
        }
    }
}
