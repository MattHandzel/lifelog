use lifelog_interface_lib::lifelog;
use std::time::Duration;
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("LIFELOG_GRPC_ADDR")
        .unwrap_or_else(|_| "http://100.118.206.104:7182".to_string());
    let token = std::env::var("LIFELOG_AUTH_TOKEN")
        .unwrap_or_else(|_| "cdf4fa2f8f324ae9afed98dfa0430755".to_string());

    println!("[CHECK] Connecting to {}...", addr);

    let channel = if addr.starts_with("https://") {
        let ca_path = std::env::var("LIFELOG_TLS_CA_CERT_PATH").unwrap_or_else(|_| {
            format!(
                "{}/.config/lifelog/tls/server-cert.pem",
                std::env::var("HOME").unwrap_or_default()
            )
        });
        println!("[CHECK] Using CA: {}", ca_path);
        let pem = std::fs::read_to_string(&ca_path)?;
        let tls = tonic::transport::ClientTlsConfig::new()
            .ca_certificate(tonic::transport::Certificate::from_pem(pem))
            .domain_name(
                std::env::var("LIFELOG_TLS_SERVER_NAME")
                    .unwrap_or_else(|_| "localhost".to_string()),
            );
        Channel::from_shared(addr.clone())?
            .tls_config(tls)?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?
    } else {
        println!("[CHECK] Using plaintext (no TLS)");
        Channel::from_shared(addr.clone())?
            .connect_timeout(Duration::from_secs(5))
            .connect()
            .await?
    };

    let token_clone = token.clone();
    let mut client = lifelog::LifelogServerServiceClient::with_interceptor(
        channel,
        move |mut req: tonic::Request<()>| {
            let val = format!("Bearer {}", token_clone);
            req.metadata_mut()
                .insert("authorization", val.parse().unwrap());
            Ok(req)
        },
    )
    .max_decoding_message_size(128 * 1024 * 1024);

    match client.get_state(lifelog::GetStateRequest {}).await {
        Ok(resp) => {
            let resp = resp.into_inner();
            println!("[OK] get_state succeeded!");
            if let Some(ss) = &resp.server_state {
                println!("  total_frames: {}", ss.total_frames_stored);
                println!("  disk_usage: {} bytes", ss.disk_usage_bytes);
            }
        }
        Err(e) => {
            eprintln!("[FAIL] get_state: {}", e);
            std::process::exit(1);
        }
    }

    match client
        .query(lifelog::QueryRequest {
            query: Some(lifelog::Query::default()),
        })
        .await
    {
        Ok(resp) => {
            let qr = resp.into_inner();
            println!("[OK] query returned {} results", qr.keys.len());
            for key in &qr.keys {
                println!("  - {}", key.uuid);
            }
        }
        Err(e) => {
            eprintln!("[FAIL] query: {}", e);
        }
    }

    println!("\n[DONE] Frontend gRPC connectivity verified!");
    Ok(())
}
