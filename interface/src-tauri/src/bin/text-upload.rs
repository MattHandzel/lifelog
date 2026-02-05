use clap::{App, Arg, SubCommand};
use config::load_config;
use lifelog_interface_lib::api_client;

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

    let _config = load_config();
    let _client = api_client::create_client();
    let _base_url = api_client::get_api_base_url();

    if let Some(matches) = matches.subcommand_matches("upload") {
        let file_path = matches.value_of("FILE").unwrap();
        
        println!("File upload functionality now requires the server to be running.");
        println!("Please use the lifelog-server-backend service with the appropriate API endpoints.");
        println!("Would have uploaded: {}", file_path);
        
    } else if let Some(matches) = matches.subcommand_matches("search") {
        let pattern = matches.value_of("PATTERN").unwrap();
        
        println!("Search functionality now requires the server to be running.");
        println!("Please use the lifelog-server-backend service with the appropriate API endpoints.");
        println!("Would have searched for: {}", pattern);
        
    } else if let Some(_) = matches.subcommand_matches("list") {
        println!("List functionality now requires the server to be running.");
        println!("Please use the lifelog-server-backend service with the appropriate API endpoints.");
        
    } else {
        println!("Please provide a valid command. Use --help for more information.");
    }

    Ok(())
} 