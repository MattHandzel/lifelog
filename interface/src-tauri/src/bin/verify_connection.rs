use std::time::Duration;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};

pub mod lifelog {
    tonic::include_proto!("lifelog");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "https://100.118.206.104:7182";
    let ca_path = "/home/matth/.config/lifelog/tls/server-ca.pem";
    let token = std::env::var("LIFELOG_AUTH_TOKEN")?;

    println!("[CHECK] Connecting to {}...", addr);
    println!("[CHECK] Using CA: {}", ca_path);

    let pem = std::fs::read_to_string(ca_path)?;
    let ca = Certificate::from_pem(pem);

    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("server.matthandzel.com");

    let channel = Channel::from_static(addr)
        .tls_config(tls)?
        .connect_timeout(Duration::from_secs(5))
        .connect()
        .await?;

    let mut client =
        lifelog::lifelog_server_service_client::LifelogServerServiceClient::with_interceptor(
            channel,
            move |mut req: tonic::Request<()>| {
                let val = format!("Bearer {}", token);
                req.metadata_mut()
                    .insert("authorization", val.parse().unwrap());
                Ok(req)
            },
        );

    match client.get_state(lifelog::GetStateRequest {}).await {
        Ok(_) => println!("[CHECK] SUCCESS: gRPC connection verified and authenticated!"),
        Err(e) => {
            eprintln!("[CHECK] FAILURE: RPC failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
