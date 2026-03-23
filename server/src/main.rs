use base64::Engine;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use lifelog_core::uuid::Uuid;
use lifelog_server::grpc_service::GRPCServerLifelogServerService;
use lifelog_server::server::Server as LifelogServer;
use lifelog_server::server::ServerHandle as LifelogServerHandle;
use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::lifelog_server_service_server::LifelogServerServiceServer;
use lifelog_types::{PairCollectorRequest, FILE_DESCRIPTOR_SET};
use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
use rustls::Error as RustlsError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tonic::metadata::MetadataValue;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Server as TonicServer};
use tonic::{Request, Status};
use tonic_reflection::server::Builder;

#[derive(Parser)]
#[command(author, version, about = "Lifelog Server Backend", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gRPC server (default)
    Serve,
    /// Generate a secure random token for authentication
    GenerateToken,
    /// Interactive first-run setup for server + config + TLS + tokens
    Init,
    /// Pair this device as a collector against a running server
    Join {
        /// Server URL, e.g. https://my-server:7182
        server_url: String,
        /// Skip interactive confirmation and use environment variables
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Debug)]
struct TofuCertVerifier {
    provider: Arc<rustls::crypto::CryptoProvider>,
}

impl TofuCertVerifier {
    fn new() -> Self {
        Self {
            provider: Arc::new(rustls::crypto::ring::default_provider()),
        }
    }
}

impl ServerCertVerifier for TofuCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, RustlsError> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[derive(Debug, Clone)]
struct OnboardingPaths {
    config_dir: PathBuf,
    config_path: PathBuf,
    env_path: PathBuf,
    server_cert_path: PathBuf,
    server_key_path: PathBuf,
    server_ca_path: PathBuf,
}

fn onboarding_paths() -> Result<OnboardingPaths, lifelog_core::LifelogError> {
    let home = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .ok_or_else(|| lifelog_core::LifelogError::Validation {
            field: "home_dir".to_string(),
            reason: "failed to resolve home directory".to_string(),
        })?;
    let config_dir = home.join(".config").join("lifelog");
    let tls_dir = config_dir.join("tls");
    Ok(OnboardingPaths {
        config_dir: config_dir.clone(),
        config_path: config_dir.join("lifelog-config.toml"),
        env_path: config_dir.join(".env"),
        server_cert_path: tls_dir.join("server-cert.pem"),
        server_key_path: tls_dir.join("server-key.pem"),
        server_ca_path: tls_dir.join("server-ca.pem"),
    })
}

fn default_device_name() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .and_then(|s| {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .or_else(|| {
            fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "device".to_string())
}

fn sanitize_collector_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if matches!(ch, '-' | '_') {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "collector".to_string()
    } else {
        out
    }
}

fn ensure_parent(path: &Path) -> Result<(), lifelog_core::LifelogError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<(), lifelog_core::LifelogError> {
    ensure_parent(path)?;
    fs::write(path, content)?;
    Ok(())
}

fn to_toml_value<T: Serialize>(value: &T) -> Result<toml::Value, lifelog_core::LifelogError> {
    let json = serde_json::to_value(value).map_err(|e| lifelog_core::LifelogError::Validation {
        field: "toml".to_string(),
        reason: format!("json conversion failed: {}", e),
    })?;
    json_to_toml_value(json)
}

fn json_to_toml_value(value: serde_json::Value) -> Result<toml::Value, lifelog_core::LifelogError> {
    match value {
        serde_json::Value::Null => Err(lifelog_core::LifelogError::Validation {
            field: "toml".to_string(),
            reason: "null is not representable in TOML".to_string(),
        }),
        serde_json::Value::Bool(v) => Ok(toml::Value::Boolean(v)),
        serde_json::Value::Number(num) => {
            if let Some(v) = num.as_i64() {
                return Ok(toml::Value::Integer(v));
            }
            if let Some(v) = num.as_f64() {
                return Ok(toml::Value::Float(v));
            }
            Err(lifelog_core::LifelogError::Validation {
                field: "toml".to_string(),
                reason: format!("invalid number: {}", num),
            })
        }
        serde_json::Value::String(v) => Ok(toml::Value::String(v)),
        serde_json::Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for v in arr {
                out.push(json_to_toml_value(v)?);
            }
            Ok(toml::Value::Array(out))
        }
        serde_json::Value::Object(map) => {
            let mut out = toml::map::Map::new();
            for (k, v) in map {
                out.insert(k, json_to_toml_value(v)?);
            }
            Ok(toml::Value::Table(out))
        }
    }
}

fn apply_storage_root_paths(cfg: &mut config::CollectorConfig, storage_root: &Path) {
    let data_dir = storage_root.join("data");
    if let Some(screen) = cfg.screen.as_mut() {
        screen.output_dir = data_dir.join("screen").display().to_string();
    }
    if let Some(browser) = cfg.browser.as_mut() {
        browser.output_file = data_dir
            .join("browser")
            .join("last_query_chrome_micros.txt")
            .display()
            .to_string();
        browser.input_file = "~/.config/google-chrome/Default/History".to_string();
        browser.browser_type = "chrome".to_string();
    }
    if let Some(camera) = cfg.camera.as_mut() {
        camera.output_dir = data_dir.join("camera").display().to_string();
    }
    if let Some(microphone) = cfg.microphone.as_mut() {
        microphone.output_dir = data_dir.join("microphone").display().to_string();
    }
    if let Some(processes) = cfg.processes.as_mut() {
        processes.output_dir = data_dir.join("processes").display().to_string();
    }
    if let Some(hyprland) = cfg.hyprland.as_mut() {
        hyprland.output_dir = data_dir.join("hyprland").display().to_string();
    }
    if let Some(weather) = cfg.weather.as_mut() {
        weather.output_dir = data_dir.join("weather").display().to_string();
    }
    if let Some(wifi) = cfg.wifi.as_mut() {
        wifi.output_dir = data_dir.join("wifi").display().to_string();
    }
    if let Some(clipboard) = cfg.clipboard.as_mut() {
        clipboard.output_dir = data_dir.join("clipboard").display().to_string();
    }
    if let Some(shell_history) = cfg.shell_history.as_mut() {
        shell_history.output_dir = data_dir.join("shell_history").display().to_string();
    }
    if let Some(mouse) = cfg.mouse.as_mut() {
        mouse.output_dir = data_dir.join("mouse").display().to_string();
    }
    if let Some(window_activity) = cfg.window_activity.as_mut() {
        window_activity.output_dir = data_dir.join("window_activity").display().to_string();
    }
    if let Some(keyboard) = cfg.keyboard.as_mut() {
        keyboard.output_dir = data_dir.join("keystrokes").display().to_string();
    }
}

fn ensure_storage_dirs(cfg: &config::CollectorConfig) -> Result<(), lifelog_core::LifelogError> {
    for path in [
        cfg.screen.as_ref().map(|v| v.output_dir.clone()),
        cfg.camera.as_ref().map(|v| v.output_dir.clone()),
        cfg.microphone.as_ref().map(|v| v.output_dir.clone()),
        cfg.processes.as_ref().map(|v| v.output_dir.clone()),
        cfg.hyprland.as_ref().map(|v| v.output_dir.clone()),
        cfg.weather.as_ref().map(|v| v.output_dir.clone()),
        cfg.wifi.as_ref().map(|v| v.output_dir.clone()),
        cfg.clipboard.as_ref().map(|v| v.output_dir.clone()),
        cfg.shell_history.as_ref().map(|v| v.output_dir.clone()),
        cfg.mouse.as_ref().map(|v| v.output_dir.clone()),
        cfg.window_activity.as_ref().map(|v| v.output_dir.clone()),
        cfg.keyboard.as_ref().map(|v| v.output_dir.clone()),
    ] {
        if let Some(dir) = path {
            fs::create_dir_all(dir)?;
        }
    }
    if let Some(browser) = cfg.browser.as_ref() {
        ensure_parent(Path::new(&browser.output_file))?;
    }
    Ok(())
}

fn init_or_load_root_config(
    config_path: &Path,
) -> Result<toml::value::Table, lifelog_core::LifelogError> {
    if !config_path.exists() {
        return Ok(toml::value::Table::new());
    }
    let raw = fs::read_to_string(config_path)?;
    let parsed: toml::Value =
        toml::from_str(&raw).map_err(|e| lifelog_core::LifelogError::Validation {
            field: "lifelog-config.toml".to_string(),
            reason: format!("failed to parse {}: {}", config_path.display(), e),
        })?;
    match parsed {
        toml::Value::Table(t) => Ok(t),
        _ => Err(lifelog_core::LifelogError::Validation {
            field: "lifelog-config.toml".to_string(),
            reason: "root value must be a TOML table".to_string(),
        }),
    }
}

fn write_unified_config(
    config_path: &Path,
    collector_cfg: &config::CollectorConfig,
    collector_id: &str,
    alias: &str,
    server_cfg: &config::ServerConfig,
) -> Result<(), lifelog_core::LifelogError> {
    let mut root = init_or_load_root_config(config_path)?;

    let mut runtime = toml::value::Table::new();
    runtime.insert(
        "collector_id".to_string(),
        toml::Value::String(collector_id.to_string()),
    );
    root.insert("runtime".to_string(), toml::Value::Table(runtime));

    let mut collectors = root
        .remove("collectors")
        .and_then(|v| v.as_table().cloned())
        .unwrap_or_default();
    collectors.insert(collector_id.to_string(), to_toml_value(collector_cfg)?);
    root.insert("collectors".to_string(), toml::Value::Table(collectors));

    root.insert("server".to_string(), to_toml_value(server_cfg)?);

    if !root.contains_key("transforms") {
        let transforms = vec![lifelog_types::TransformSpec {
            id: "ocr".to_string(),
            enabled: true,
            source_origin: "*:screen".to_string(),
            language: Some("eng".to_string()),
            transform_type: String::new(),
            service_endpoint: String::new(),
            params: Default::default(),
            priority: 0,
            destination_modality: String::new(),
        }];
        root.insert("transforms".to_string(), to_toml_value(&transforms)?);
    }

    let mut aliases = root
        .remove("device_aliases")
        .or_else(|| root.remove("deviceAliases"))
        .and_then(|v| v.as_table().cloned())
        .unwrap_or_default();
    aliases.insert(
        collector_id.to_string(),
        toml::Value::String(alias.to_string()),
    );
    root.insert("device_aliases".to_string(), toml::Value::Table(aliases));

    let rendered = toml::to_string_pretty(&toml::Value::Table(root)).map_err(|e| {
        lifelog_core::LifelogError::Validation {
            field: "lifelog-config.toml".to_string(),
            reason: format!("failed to render config TOML: {}", e),
        }
    })?;
    write_file(config_path, &rendered)
}

fn write_env_file(
    env_path: &Path,
    auth_token: &str,
    enrollment_token: &str,
    cert_path: &Path,
    key_path: &Path,
    config_path: &Path,
) -> Result<(), lifelog_core::LifelogError> {
    let content = format!(
        "LIFELOG_AUTH_TOKEN={}\nLIFELOG_ENROLLMENT_TOKEN={}\nLIFELOG_TLS_CERT_PATH={}\nLIFELOG_TLS_KEY_PATH={}\nLIFELOG_CONFIG_PATH={}\n",
        auth_token,
        enrollment_token,
        cert_path.display(),
        key_path.display(),
        config_path.display(),
    );
    write_file(env_path, &content)
}

fn der_to_pem(der: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap_or_default());
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}

fn sha256_fingerprint(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

fn generate_self_signed_cert(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(), lifelog_core::LifelogError> {
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = rcgen::generate_simple_self_signed(subject_alt_names).map_err(|e| {
        lifelog_core::LifelogError::Validation {
            field: "tls".to_string(),
            reason: format!("certificate generation failed: {}", e),
        }
    })?;
    write_file(cert_path, cert.cert.pem().as_str())?;
    write_file(key_path, cert.key_pair.serialize_pem().as_str())?;
    Ok(())
}

fn parse_server_url(server_url: &str) -> Result<(String, String, u16), lifelog_core::LifelogError> {
    if !server_url.starts_with("https://") {
        return Err(lifelog_core::LifelogError::Validation {
            field: "server_url".to_string(),
            reason: "must begin with https://".to_string(),
        });
    }
    let without_scheme = server_url.trim_start_matches("https://");
    let host_port = without_scheme.split('/').next().unwrap_or_default();
    if host_port.is_empty() {
        return Err(lifelog_core::LifelogError::Validation {
            field: "server_url".to_string(),
            reason: "missing host".to_string(),
        });
    }
    let (host, port) = if let Some((h, p)) = host_port.rsplit_once(':') {
        let parsed = p
            .parse::<u16>()
            .map_err(|_| lifelog_core::LifelogError::Validation {
                field: "server_url".to_string(),
                reason: "invalid port".to_string(),
            })?;
        (h.to_string(), parsed)
    } else {
        (host_port.to_string(), 443)
    };

    let normalized = format!("https://{}:{}", host, port);
    Ok((normalized, host, port))
}

async fn fetch_server_cert_fingerprint(
    host: &str,
    port: u16,
) -> Result<(Vec<u8>, String), lifelog_core::LifelogError> {
    let stream = TcpStream::connect((host, port)).await?;
    let rustls_cfg = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(TofuCertVerifier::new()))
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(rustls_cfg));
    let server_name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|_| {
        lifelog_core::LifelogError::Validation {
            field: "server_url".to_string(),
            reason: "invalid DNS name".to_string(),
        }
    })?;
    let tls_stream = connector.connect(server_name, stream).await?;
    let certs = tls_stream.get_ref().1.peer_certificates().ok_or_else(|| {
        lifelog_core::LifelogError::Validation {
            field: "server_cert".to_string(),
            reason: "server did not present certificates".to_string(),
        }
    })?;
    let cert = certs
        .first()
        .ok_or_else(|| lifelog_core::LifelogError::Validation {
            field: "server_cert".to_string(),
            reason: "empty server certificate chain".to_string(),
        })?
        .as_ref()
        .to_vec();
    let fp = sha256_fingerprint(cert.as_slice());
    Ok((cert, fp))
}

async fn pair_collector(
    server_url: &str,
    server_host: &str,
    enrollment_token: &str,
    client_hint: &str,
    ca_cert_pem: &str,
) -> Result<String, lifelog_core::LifelogError> {
    let tls = ClientTlsConfig::new()
        .domain_name(server_host.to_string())
        .ca_certificate(Certificate::from_pem(ca_cert_pem));
    let channel: Channel = Endpoint::from_shared(server_url.to_string())?
        .tls_config(tls)
        .map_err(|e| lifelog_core::LifelogError::Validation {
            field: "server_url".to_string(),
            reason: format!("TLS config error: {}", e),
        })?
        .connect()
        .await?;

    let token = enrollment_token.to_string();
    let interceptor = move |mut req: Request<()>| -> Result<Request<()>, Status> {
        let bearer = format!("Bearer {}", token);
        let value = MetadataValue::try_from(bearer.as_str())
            .map_err(|_| Status::unauthenticated("invalid enrollment token format"))?;
        req.metadata_mut().insert("authorization", value);
        Ok(req)
    };
    let mut client = LifelogServerServiceClient::with_interceptor(channel, interceptor);

    let mut req = Request::new(PairCollectorRequest {
        enrollment_token: enrollment_token.to_string(),
    });
    let client_id = MetadataValue::try_from(client_hint).map_err(|_| {
        lifelog_core::LifelogError::Validation {
            field: "client_hint".to_string(),
            reason: "contains invalid metadata characters".to_string(),
        }
    })?;
    req.metadata_mut().insert("x-lifelog-client-id", client_id);
    let resp = client.pair_collector(req).await?;
    Ok(resp.into_inner().collector_id)
}

async fn run_init() -> Result<(), lifelog_core::LifelogError> {
    let paths = onboarding_paths()?;
    fs::create_dir_all(paths.config_dir.clone())?;

    let overwrite = if paths.config_path.exists() || paths.env_path.exists() {
        inquire::Confirm::new("Existing onboarding files found. Overwrite them?")
            .with_default(false)
            .prompt()
            .map_err(|e| lifelog_core::LifelogError::Validation {
                field: "prompt".to_string(),
                reason: e.to_string(),
            })?
    } else {
        true
    };
    if !overwrite {
        return Err(lifelog_core::LifelogError::Validation {
            field: "init".to_string(),
            reason: "aborted: onboarding files already exist".to_string(),
        });
    }

    let alias = inquire::Text::new("Device alias")
        .with_default(default_device_name().as_str())
        .prompt()
        .map_err(|e| lifelog_core::LifelogError::Validation {
            field: "device_alias".to_string(),
            reason: e.to_string(),
        })?;
    let collector_id = sanitize_collector_id(alias.as_str());

    let default_storage = directories::BaseDirs::new()
        .map(|d| d.home_dir().join("lifelog").display().to_string())
        .unwrap_or_else(|| "/tmp/lifelog".to_string());
    let storage_root = inquire::Text::new("Storage root directory")
        .with_default(default_storage.as_str())
        .prompt()
        .map_err(|e| lifelog_core::LifelogError::Validation {
            field: "storage_root".to_string(),
            reason: e.to_string(),
        })?;

    let server_host = inquire::Text::new("Server bind host")
        .with_default("0.0.0.0")
        .prompt()
        .map_err(|e| lifelog_core::LifelogError::Validation {
            field: "server_host".to_string(),
            reason: e.to_string(),
        })?;
    let server_port_input = inquire::Text::new("Server gRPC port")
        .with_default("7182")
        .prompt()
        .map_err(|e| lifelog_core::LifelogError::Validation {
            field: "server_port".to_string(),
            reason: e.to_string(),
        })?;
    let server_port: u32 =
        server_port_input
            .parse()
            .map_err(|_| lifelog_core::LifelogError::Validation {
                field: "server_port".to_string(),
                reason: "must be a valid u32".to_string(),
            })?;
    let enable_microphone = inquire::Confirm::new("Enable microphone capture?")
        .with_default(false)
        .prompt()
        .map_err(|e| lifelog_core::LifelogError::Validation {
            field: "microphone".to_string(),
            reason: e.to_string(),
        })?;

    let storage_root_path = PathBuf::from(storage_root);
    fs::create_dir_all(storage_root_path.join("data"))?;
    ensure_parent(paths.server_cert_path.as_path())?;
    generate_self_signed_cert(
        paths.server_cert_path.as_path(),
        paths.server_key_path.as_path(),
    )?;

    let cert_pem = fs::read_to_string(paths.server_cert_path.clone())?;
    let cert_fp = sha256_fingerprint(cert_pem.as_bytes());

    let auth_token = Uuid::new_v4().to_string().replace('-', "");
    let enrollment_token = Uuid::new_v4().to_string().replace('-', "");

    let mut collector_cfg = config::create_default_config();
    collector_cfg.id = collector_id.clone();
    collector_cfg.host = "127.0.0.1".to_string();
    collector_cfg.port = 7190;
    if let Some(mic) = collector_cfg.microphone.as_mut() {
        mic.enabled = enable_microphone;
    }
    apply_storage_root_paths(&mut collector_cfg, storage_root_path.as_path());
    ensure_storage_dirs(&collector_cfg)?;

    let mut server_cfg = config::default_server_config();
    server_cfg.host = server_host;
    server_cfg.port = server_port;
    server_cfg.cas_path = storage_root_path.join("cas").display().to_string();
    fs::create_dir_all(PathBuf::from(&server_cfg.cas_path))?;

    write_unified_config(
        paths.config_path.as_path(),
        &collector_cfg,
        collector_id.as_str(),
        alias.as_str(),
        &server_cfg,
    )?;
    write_env_file(
        paths.env_path.as_path(),
        auth_token.as_str(),
        enrollment_token.as_str(),
        paths.server_cert_path.as_path(),
        paths.server_key_path.as_path(),
        paths.config_path.as_path(),
    )?;

    println!("Lifelog init complete.");
    println!("Config: {}", paths.config_path.display());
    println!("Env: {}", paths.env_path.display());
    println!("TLS cert: {}", paths.server_cert_path.display());
    println!("TLS key: {}", paths.server_key_path.display());
    println!("Enrollment token: {}", enrollment_token);
    println!("Server cert SHA256 fingerprint: {}", cert_fp);
    println!(
        "Next: source {} and run `lifelog-server serve`",
        paths.env_path.display()
    );
    Ok(())
}

async fn run_join(server_url: String, yes: bool) -> Result<(), lifelog_core::LifelogError> {
    let (normalized_url, host, port) = parse_server_url(server_url.as_str())?;
    let paths = onboarding_paths()?;
    fs::create_dir_all(paths.config_dir.clone())?;

    let (server_cert_der, fingerprint) = fetch_server_cert_fingerprint(host.as_str(), port).await?;
    println!(
        "Server certificate SHA256 fingerprint for {} is:\n{}",
        normalized_url, fingerprint
    );

    if !yes {
        let trust = inquire::Confirm::new("Trust this server certificate and continue pairing?")
            .with_default(false)
            .prompt()
            .map_err(|e| lifelog_core::LifelogError::Validation {
                field: "join_confirm".to_string(),
                reason: e.to_string(),
            })?;
        if !trust {
            return Err(lifelog_core::LifelogError::Validation {
                field: "join".to_string(),
                reason: "aborted: certificate not trusted".to_string(),
            });
        }
    } else {
        println!("Non-interactive mode: Trusting certificate automatically.");
    }

    let server_cert_pem = der_to_pem(server_cert_der.as_slice());
    write_file(paths.server_ca_path.as_path(), server_cert_pem.as_str())?;

    let enrollment_token = if yes {
        std::env::var("LIFELOG_ENROLLMENT_TOKEN").map_err(|_| {
            lifelog_core::LifelogError::Validation {
                field: "enrollment_token".to_string(),
                reason: "LIFELOG_ENROLLMENT_TOKEN env var must be set in non-interactive mode"
                    .to_string(),
            }
        })?
    } else {
        inquire::Password::new("Enrollment token")
            .without_confirmation()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()
            .map_err(|e| lifelog_core::LifelogError::Validation {
                field: "enrollment_token".to_string(),
                reason: e.to_string(),
            })?
    };

    let client_hint_default = sanitize_collector_id(default_device_name().as_str());
    let (client_hint, alias) = if yes {
        (client_hint_default.clone(), client_hint_default)
    } else {
        let client_hint = inquire::Text::new("Collector client hint")
            .with_default(client_hint_default.as_str())
            .prompt()
            .map_err(|e| lifelog_core::LifelogError::Validation {
                field: "client_hint".to_string(),
                reason: e.to_string(),
            })?;
        let alias = inquire::Text::new("Collector alias")
            .with_default(client_hint.as_str())
            .prompt()
            .map_err(|e| lifelog_core::LifelogError::Validation {
                field: "collector_alias".to_string(),
                reason: e.to_string(),
            })?;
        (client_hint, alias)
    };

    let paired_id = pair_collector(
        normalized_url.as_str(),
        host.as_str(),
        enrollment_token.as_str(),
        client_hint.as_str(),
        server_cert_pem.as_str(),
    )
    .await?;

    let mut collector_cfg = config::create_default_config();
    collector_cfg.id = paired_id.clone();
    let data_root = directories::BaseDirs::new()
        .map(|d| d.home_dir().join("lifelog"))
        .unwrap_or_else(|| PathBuf::from("/tmp/lifelog"));
    apply_storage_root_paths(&mut collector_cfg, data_root.as_path());
    ensure_storage_dirs(&collector_cfg)?;

    let mut server_cfg = config::default_server_config();
    server_cfg.host = "0.0.0.0".to_string();
    let existing_auth = std::env::var("LIFELOG_AUTH_TOKEN").ok();
    let auth_token = existing_auth.unwrap_or_else(|| Uuid::new_v4().to_string().replace('-', ""));

    write_unified_config(
        paths.config_path.as_path(),
        &collector_cfg,
        paired_id.as_str(),
        alias.as_str(),
        &server_cfg,
    )?;

    let env_content = format!(
        "LIFELOG_CONFIG_PATH={}\nLIFELOG_AUTH_TOKEN={}\nLIFELOG_TLS_CA_CERT_PATH={}\n",
        paths.config_path.display(),
        auth_token,
        paths.server_ca_path.display(),
    );
    write_file(paths.env_path.as_path(), env_content.as_str())?;

    println!("Collector paired successfully.");
    println!("Collector ID: {}", paired_id);
    println!("Saved CA cert: {}", paths.server_ca_path.display());
    println!("Updated config: {}", paths.config_path.display());
    println!("Collector env: {}", paths.env_path.display());
    println!(
        "Next: source {} and run `lifelog-collector --server-address {}`",
        paths.env_path.display(),
        normalized_url
    );
    Ok(())
}

fn check_disk_space(path: &str) {
    let output = std::process::Command::new("df")
        .arg("-B1")
        .arg(path)
        .output();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        if let Some(line) = stdout.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(avail) = parts[3].parse::<u64>() {
                    if avail < 1_073_741_824 {
                        tracing::warn!(
                            path = %path,
                            available_bytes = avail,
                            "CAS directory has less than 1 GB of free disk space"
                        );
                    }
                }
            }
        }
    }
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut config = config::load_server_config_from_unified().unwrap_or_else(|| {
        panic!(
            "Missing or invalid [server] in {}. No defaults are applied.",
            std::env::var("LIFELOG_CONFIG_PATH")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| config::default_lifelog_config_path())
                .display()
        )
    });
    if let Ok(host) = std::env::var("LIFELOG_HOST") {
        config.host = host;
    }
    if let Ok(port) = std::env::var("LIFELOG_PORT") {
        if let Ok(p) = port.parse() {
            config.port = p;
        }
    }
    if let Ok(db) = std::env::var("LIFELOG_DB_ENDPOINT") {
        config.database_endpoint = db;
    }
    if let Ok(cas) = std::env::var("LIFELOG_CAS_PATH") {
        config.cas_path = cas;
    }
    let server = LifelogServer::new(&config).await?;
    check_disk_space(&config.cas_path);

    let addr = format!("{}:{}", config.host, config.port).parse()?;

    tracing::info!("Starting server on {}", addr);
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    let _time: DateTime<Utc> = Utc::now();
    let _uuid = Uuid::new_v4();

    let server_handle =
        LifelogServerHandle::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));
    let server_handle2 = server_handle.clone();

    let health_handle = server_handle.clone();
    let health_reporter_bg = health_reporter.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            let server = health_handle.server.read().await;
            let status = match server.postgres_pool.get().await {
                Ok(client) => match client.execute("SELECT 1", &[]).await {
                    Ok(_) => tonic_health::ServingStatus::Serving,
                    Err(e) => {
                        tracing::warn!(error = %e, "health check: postgres query failed");
                        tonic_health::ServingStatus::NotServing
                    }
                },
                Err(e) => {
                    tracing::warn!(error = %e, "health check: postgres pool unavailable");
                    tonic_health::ServingStatus::NotServing
                }
            };
            health_reporter_bg.set_service_status("", status).await;
        }
    });

    let loop_handle = server_handle.clone();
    tokio::task::spawn(async move {
        loop_handle.r#loop().await;
    });

    let retention_handle = server_handle.clone();
    tokio::task::spawn(async move {
        let interval_secs = std::env::var("LIFELOG_RETENTION_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(86_400)
            .max(60);
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            match retention_handle.run_retention_once().await {
                Ok(summary) => {
                    if summary.deleted_records > 0 || summary.deleted_blobs > 0 {
                        tracing::info!(
                            deleted_records = summary.deleted_records,
                            deleted_blobs = summary.deleted_blobs,
                            "retention pruncompleted"
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "retention prune failed");
                }
            }
        }
    });

    let summary_handle = server_handle.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86400));
        interval.tick().await;
        loop {
            interval.tick().await;
            let server = summary_handle.server.read().await;
            let endpoint = std::env::var("LIFELOG_OLLAMA_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = std::env::var("LIFELOG_SUMMARY_MODEL")
                .unwrap_or_else(|_| "gemma3:4b-it-qat".to_string());
            match lifelog_server::transform::summary::generate_daily_summary(
                &server.postgres_pool,
                &server.cas,
                &server.http_client,
                &endpoint,
                &model,
            )
            .await
            {
                Ok(()) => {
                    tracing::info!("daily summary generated successfully");
                }
                Err(e) => {
                    tracing::error!(error = %e, "daily summary generation failed");
                }
            }
        }
    });

    let meeting_handle = server_handle.clone();
    tokio::task::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        interval.tick().await;
        loop {
            interval.tick().await;
            let server = meeting_handle.server.read().await;
            match lifelog_server::transform::meeting::detect_meetings(&server.postgres_pool).await {
                Ok(()) => {
                    tracing::debug!("meeting detection completed");
                }
                Err(e) => {
                    tracing::error!(error = %e, "meeting detection failed");
                }
            }
        }
    });

    let deploy_config = config::load_server_deploy_config();
    let tls_config = deploy_config.tls;
    let mut builder = TonicServer::builder()
        .accept_http1(true)
        .layer(tonic_web::GrpcWebLayer::new());

    if !tls_config.is_enabled() && !deploy_config.allow_plaintext {
        return Err(lifelog_core::LifelogError::Validation {
            field: "tls_cert_path/tls_key_path".to_string(),
            reason: "must both be set in [server] config or as env vars. \
                     Plaintext gRPC is not allowed for security reasons. \
                     Set allow_plaintext = true in config or LIFELOG_ALLOW_PLAINTEXT=1 for trusted networks. \
                     See docs/SETUP_TLS.md for how to generate certificates."
                .to_string(),
        }
        .into());
    }

    if !tls_config.is_enabled() {
        tracing::warn!("Running WITHOUT TLS (allow_plaintext=true). Only safe on encrypted networks like Tailscale.");
    }

    if let (Some(cert_path), Some(key_path)) = (&tls_config.cert_path, &tls_config.key_path) {
        let cert = std::fs::read_to_string(cert_path).map_err(|e| {
            format!(
                "Failed to read certificate at {}: {}. Ensure the path is correct and accessible.",
                cert_path, e
            )
        })?;
        let key = std::fs::read_to_string(key_path).map_err(|e| {
            format!(
                "Failed to read private key at {}: {}. Ensure the path is correct and accessible.",
                key_path, e
            )
        })?;
        let identity = tonic::transport::Identity::from_pem(cert, key);
        let tls = tonic::transport::ServerTlsConfig::new().identity(identity);
        builder = builder.tls_config(tls)?;
        tracing::info!(cert = %cert_path, key = %key_path, "TLS enabled");
    }

    let _auth_token = std::env::var("LIFELOG_AUTH_TOKEN").map_err(|_| {
        lifelog_core::LifelogError::Validation {
            field: "LIFELOG_AUTH_TOKEN".to_string(),
            reason: "must be set. This token is required to authenticate connected collectors."
                .to_string(),
        }
    })?;
    let _enrollment_token = std::env::var("LIFELOG_ENROLLMENT_TOKEN").map_err(|_| {
        lifelog_core::LifelogError::Validation {
            field: "LIFELOG_ENROLLMENT_TOKEN".to_string(),
            reason: "must be set. This token is used for initial pairing of new devices."
                .to_string(),
        }
    })?;

    builder
        .add_service(reflection_service)
        .add_service(health_service)
        .add_service(
            LifelogServerServiceServer::new(GRPCServerLifelogServerService {
                server: server_handle2,
            })
            .max_decoding_message_size(128 * 1024 * 1024)
            .max_encoding_message_size(128 * 1024 * 1024),
        )
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Install default crypto provider for rustls 0.23+
    let _ = rustls::crypto::ring::default_provider().install_default();
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Serve) {
        Commands::Serve => run_server().await,
        Commands::GenerateToken => {
            let auth_token = Uuid::new_v4().to_string().replace("-", "");
            let enrollment_token = Uuid::new_v4().to_string().replace("-", "");
            println!("Generated LIFELOG_AUTH_TOKEN: {}", auth_token);
            println!("Generated LIFELOG_ENROLLMENT_TOKEN: {}", enrollment_token);
            Ok(())
        }
        Commands::Init => run_init().await.map_err(Into::into),
        Commands::Join { server_url, yes } => run_join(server_url, yes).await.map_err(Into::into),
    }
}
