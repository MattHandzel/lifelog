use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::{lifelog_data::Payload, GetDataRequest, LifelogDataKey};
use std::path::Path;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Get the latest UUID from Postgres
    let output = std::process::Command::new("psql")
        .args([
            "-d",
            "lifelog",
            "-t",
            "-c",
            "SELECT id FROM screen_records ORDER BY t_canonical DESC LIMIT 1;",
        ])
        .output()?;
    let uuid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uuid_str.is_empty() {
        println!("No screenshots found in database.");
        return Ok(());
    }
    println!("Fetching latest screenshot UUID: {}", uuid_str);

    // 2. Setup gRPC client with TLS
    let cert_path = "cert.pem";
    let ca_pem = std::fs::read_to_string(cert_path)?;
    let tls = ClientTlsConfig::new()
        .domain_name("server.matthandzel.com")
        .ca_certificate(Certificate::from_pem(ca_pem));

    let channel = Channel::from_static("https://localhost:7182")
        .tls_config(tls)?
        .connect()
        .await?;

    // Add auth token from env
    let token = std::env::var("LIFELOG_AUTH_TOKEN")?;
    let mut client =
        LifelogServerServiceClient::with_interceptor(channel, move |mut req: tonic::Request<()>| {
            let token_str = format!("Bearer {}", token);
            req.metadata_mut()
                .insert("authorization", token_str.parse().unwrap());
            Ok(req)
        });

    // 3. Call GetData
    let req = GetDataRequest {
        keys: vec![LifelogDataKey {
            uuid: uuid_str,
            origin: "matts-laptop:screen".to_string(), // Adjust if needed
        }],
    };

    let resp = client.get_data(req).await?;
    let data_list = resp.into_inner().data;

    if let Some(data) = data_list.first() {
        if let Some(Payload::Screenframe(frame)) = &data.payload {
            println!("SUCCESS: Received screen frame!");
            println!("Resolution: {}x{}", frame.width, frame.height);
            println!("Image bytes size: {} KB", frame.image_bytes.len() / 1024);
        } else {
            println!(
                "Received data but it was not a screen frame: {:?}",
                data.payload
            );
        }
    } else {
        println!("Server returned empty data list.");
    }

    Ok(())
}
