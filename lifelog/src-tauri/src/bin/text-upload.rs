use clap::{App, Arg, SubCommand};
use lifelog_interface_lib::config::load_config;
use lifelog_interface_lib::modules::text_upload;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = App::new("Lifelog Text Uploader")
        .version("0.1.0")
        .author("Lifelog Team")
        .about("Upload and manage text files in your lifelog")
        .subcommand(
            SubCommand::with_name("upload")
                .about("Upload a text file")
                .arg(
                    Arg::with_name("FILE")
                        .help("The file to upload")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("search")
                .about("Search for uploaded text files")
                .arg(
                    Arg::with_name("PATTERN")
                        .help("The search pattern")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("list").about("List all uploaded text files"))
        .get_matches();

    let config = load_config();

    if let Some(matches) = matches.subcommand_matches("upload") {
        let file_path = matches.value_of("FILE").unwrap();
        let result = text_upload::upload_file(&config.text_upload, Path::new(file_path)).await?;
        println!("File uploaded successfully:");
        println!("  Filename: {}", result.filename);
        println!("  File type: {}", result.file_type);
        println!("  File size: {} bytes", result.file_size);
        println!("  Stored at: {}", result.stored_path);
        println!("  Hash: {}", result.content_hash);
    } else if let Some(matches) = matches.subcommand_matches("search") {
        let pattern = matches.value_of("PATTERN").unwrap();
        let results = text_upload::search_by_filename(&config.text_upload, pattern)?;
        
        if results.is_empty() {
            println!("No matching files found.");
        } else {
            println!("Found {} matching files:", results.len());
            for (i, file) in results.iter().enumerate() {
                println!("{}. {}", i + 1, file.filename);
                println!("   Type: {}", file.file_type);
                println!("   Size: {} bytes", file.file_size);
                println!("   Path: {}", file.stored_path);
                println!();
            }
        }
    } else if let Some(_) = matches.subcommand_matches("list") {
        let results = text_upload::get_all_files(&config.text_upload)?;
        
        if results.is_empty() {
            println!("No files have been uploaded yet.");
        } else {
            println!("All uploaded files ({}):", results.len());
            for (i, file) in results.iter().enumerate() {
                println!("{}. {}", i + 1, file.filename);
                println!("   Type: {}", file.file_type);
                println!("   Size: {} bytes", file.file_size);
                println!("   Path: {}", file.stored_path);
                println!();
            }
        }
    } else {
        println!("Please provide a valid command. Use --help for more information.");
    }

    Ok(())
} 