use rustls::crypto::ring::default_provider;
pub mod assertions;
pub mod device_client;
pub mod event_gen;
pub mod fault_layer;
pub mod simulated_modalities;

use config::ServerConfig;
use fault_layer::{FaultController, FaultInjectionLayer};
use lifelog_server::grpc_service::GRPCServerLifelogServerService;
use lifelog_server::server::{Server, ServerHandle};
use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::lifelog_server_service_server::LifelogServerServiceServer;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tonic::metadata::{Ascii, MetadataValue};
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity, ServerTlsConfig};
use utils::cas::FsCas;

use device_client::DeviceClient;

fn generate_test_tls_materials(dir: &std::path::Path) -> (PathBuf, PathBuf, String, String) {
    let cert_path = dir.join("test-cert.pem");
    let key_path = dir.join("test-key.pem");
    let status = std::process::Command::new("openssl")
        .args([
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-sha256",
            "-days",
            "3650",
            "-nodes",
            "-keyout",
            key_path.to_str().expect("key path utf8"),
            "-out",
            cert_path.to_str().expect("cert path utf8"),
            "-subj",
            "/CN=localhost",
            "-addext",
            "subjectAltName=DNS:localhost,IP:127.0.0.1",
            "-addext",
            "basicConstraints=critical,CA:FALSE",
            "-addext",
            "keyUsage=critical,digitalSignature,keyEncipherment",
            "-addext",
            "extendedKeyUsage=serverAuth",
        ])
        .status()
        .expect("openssl must be available for integration tests");
    assert!(status.success(), "failed to generate test TLS materials");
    let cert_pem = std::fs::read_to_string(&cert_path).expect("read generated cert");
    let key_pem = std::fs::read_to_string(&key_path).expect("read generated key");
    (cert_path, key_path, cert_pem, key_pem)
}

#[derive(Clone)]
pub struct ClientAuthInterceptor {
    token: MetadataValue<Ascii>,
}

impl ClientAuthInterceptor {
    pub fn new(token: &str) -> Self {
        let bearer = format!("Bearer {token}");
        let token = MetadataValue::try_from(bearer.as_str()).expect("valid auth metadata");
        Self { token }
    }
}

impl Interceptor for ClientAuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request
            .metadata_mut()
            .insert("authorization", self.token.clone());
        Ok(request)
    }
}

#[derive(Clone)]
struct ServerAuthInterceptor {
    auth_token: String,
    enrollment_token: String,
}

impl Interceptor for ServerAuthInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        match request.metadata().get("authorization") {
            Some(t) => {
                let token_str = t.to_str().unwrap_or_default();
                if token_str == format!("Bearer {}", self.auth_token)
                    || token_str == format!("Bearer {}", self.enrollment_token)
                {
                    Ok(request)
                } else {
                    Err(tonic::Status::unauthenticated("Invalid token"))
                }
            }
            None => Err(tonic::Status::unauthenticated("No token provided")),
        }
    }
}

pub type TestClient =
    LifelogServerServiceClient<InterceptedService<Channel, ClientAuthInterceptor>>;

#[allow(dead_code)]
pub struct TestContext {
    pub server_addr: String,
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub client: TestClient,
    pub raw_client: LifelogServerServiceClient<Channel>,
    raw_channel: Channel,
    pub fault_controller: FaultController,
    pub cas_path: PathBuf,
    tls_ca_path: PathBuf,
    server_port: u16,
    pub pg_url: String,
    _pg_container: Option<ContainerAsync<Postgres>>,
}

struct PgSetup {
    admin_url: String,
    test_db_url_fn: Box<dyn FnOnce(&str) -> String>,
    container: Option<ContainerAsync<Postgres>>,
}

fn replace_dbname(conn_str: &str, new_dbname: &str) -> String {
    if conn_str.contains("dbname=") {
        conn_str
            .split_whitespace()
            .map(|part| {
                if part.starts_with("dbname=") {
                    format!("dbname={new_dbname}")
                } else {
                    part.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        format!("{conn_str} dbname={new_dbname}")
    }
}

async fn provision_postgres() -> PgSetup {
    if let Ok(url) = std::env::var("LIFELOG_TEST_PG_URL") {
        let base = url.clone();
        return PgSetup {
            admin_url: url,
            test_db_url_fn: Box::new(move |db_name| replace_dbname(&base, db_name)),
            container: None,
        };
    }

    if let Ok(url) = std::env::var("LIFELOG_POSTGRES_INGEST_URL") {
        let base = url.clone();
        return PgSetup {
            admin_url: url,
            test_db_url_fn: Box::new(move |db_name| replace_dbname(&base, db_name)),
            container: None,
        };
    }

    let local_url = "host=/run/postgresql dbname=postgres";
    let local_try = tokio_postgres::connect(local_url, tokio_postgres::NoTls).await;
    if local_try.is_ok() {
        let base = local_url.to_string();
        return PgSetup {
            admin_url: base.clone(),
            test_db_url_fn: Box::new(move |db_name| replace_dbname(&base, db_name)),
            container: None,
        };
    }

    eprintln!("No local PostgreSQL found, starting testcontainer...");
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start PostgreSQL testcontainer (is Docker running?)");
    let host_port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get testcontainer port");
    let admin_url =
        format!("host=127.0.0.1 port={host_port} user=postgres password=postgres dbname=postgres");
    PgSetup {
        admin_url,
        test_db_url_fn: Box::new(move |db_name| {
            format!(
                "host=127.0.0.1 port={host_port} user=postgres password=postgres dbname={db_name}"
            )
        }),
        container: Some(container),
    }
}

impl TestContext {
    #[allow(dead_code)]
    pub async fn new() -> Self {
        Self::new_with_faults(FaultController::new()).await
    }

    pub async fn new_with_faults(fault_controller: FaultController) -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let server_port = portpicker::pick_unused_port().expect("No ports available");

        let server_addr = format!("https://localhost:{}", server_port);
        let cas_path = temp_dir.path().join("cas");
        let (tls_cert_path, _tls_key_path, tls_cert_pem, tls_key_pem) =
            generate_test_tls_materials(temp_dir.path());

        std::env::set_var("LIFELOG_AUTH_TOKEN", "test-auth-token");
        std::env::set_var("LIFELOG_ENROLLMENT_TOKEN", "test-enrollment-token");
        std::env::set_var("LIFELOG_TLS_CA_CERT_PATH", &tls_cert_path);

        let test_db_name = std::env::var("LIFELOG_TEST_DB")
            .unwrap_or_else(|_| format!("lifelog_test_{}", server_port));
        let pg_setup = provision_postgres().await;
        {
            let (pg_client, pg_conn) =
                tokio_postgres::connect(&pg_setup.admin_url, tokio_postgres::NoTls)
                    .await
                    .expect("Failed to connect to postgres for test DB setup");
            tokio::spawn(pg_conn);
            let _ = pg_client
                .execute(&format!("DROP DATABASE IF EXISTS \"{test_db_name}\""), &[])
                .await;
            pg_client
                .execute(&format!("CREATE DATABASE \"{test_db_name}\""), &[])
                .await
                .expect("Failed to create test database");
        }
        let pg_test_url = (pg_setup.test_db_url_fn)(&test_db_name);
        std::env::set_var("LIFELOG_POSTGRES_INGEST_URL", &pg_test_url);

        let mut transforms = Vec::new();
        if let Ok(v) = std::env::var("LIFELOG_TRANSFORMS_JSON") {
            if !v.trim().is_empty() {
                transforms = serde_json::from_str(&v).unwrap_or_default();
            }
        }

        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: server_port as u32,
            database_endpoint: pg_test_url.clone(),
            database_name: "test_db".to_string(),
            server_name: "TestServer".to_string(),
            cas_path: cas_path.display().to_string(),
            default_correlation_window_ms: 30_000,
            retention_policy_days: std::collections::HashMap::new(),
            transforms,
        };

        let server = timeout(Duration::from_secs(30), Server::new(&config))
            .await
            .expect("Timed out creating server (check DB connectivity / schema init)")
            .expect("Failed to create server");
        let server_handle = Arc::new(RwLock::new(server));
        let handle_clone = ServerHandle::new(server_handle.clone());
        let grpc_service = GRPCServerLifelogServerService {
            server: handle_clone.clone(),
        };

        tokio::spawn(async move {
            handle_clone.r#loop().await;
        });

        let addr = format!("127.0.0.1:{}", server_port).parse().unwrap();
        let fault_layer = FaultInjectionLayer::new(fault_controller.clone());
        let server_tls =
            ServerTlsConfig::new().identity(Identity::from_pem(tls_cert_pem.clone(), tls_key_pem));
        let auth_layer = ServerAuthInterceptor {
            auth_token: "test-auth-token".to_string(),
            enrollment_token: "test-enrollment-token".to_string(),
        };

        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .layer(fault_layer)
                .tls_config(server_tls)
                .expect("set server tls config")
                .add_service(LifelogServerServiceServer::with_interceptor(
                    grpc_service,
                    auth_layer,
                ))
                .serve(addr)
                .await
                .expect("Server failed");
        });

        sleep(Duration::from_millis(250)).await;

        let tls_config = ClientTlsConfig::new()
            .domain_name("localhost")
            .ca_certificate(Certificate::from_pem(tls_cert_pem));

        let channel = timeout(
            Duration::from_secs(30),
            Channel::from_shared(server_addr.clone())
                .expect("invalid server url")
                .tls_config(tls_config)
                .expect("set client tls config")
                .connect(),
        )
        .await
        .expect("Timed out connecting to gRPC server (it may not have started)")
        .expect("Failed to connect to server");
        let raw_channel = channel.clone();
        let raw_client = LifelogServerServiceClient::new(raw_channel.clone());
        let client = LifelogServerServiceClient::with_interceptor(
            channel,
            ClientAuthInterceptor::new("test-auth-token"),
        );

        Self {
            server_addr,
            temp_dir,
            client,
            raw_client,
            raw_channel,
            fault_controller,
            cas_path,
            tls_ca_path: tls_cert_path,
            server_port,
            pg_url: pg_test_url,
            _pg_container: pg_setup.container,
        }
    }

    pub fn client(&self) -> TestClient {
        self.client.clone()
    }

    pub fn raw_client(&self) -> LifelogServerServiceClient<Channel> {
        self.raw_client.clone()
    }

    pub fn client_with_token(&self, token: &str) -> TestClient {
        LifelogServerServiceClient::with_interceptor(
            self.raw_channel.clone(),
            ClientAuthInterceptor::new(token),
        )
    }

    #[allow(dead_code)]
    pub fn create_device_clients(&self, n: usize) -> Vec<DeviceClient> {
        (0..n)
            .map(|i| DeviceClient::new(format!("device-{i}"), self.client()))
            .collect()
    }

    #[allow(dead_code)]
    pub fn cas(&self) -> FsCas {
        FsCas::new(&self.cas_path)
    }

    #[allow(dead_code)]
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn tls_ca_path(&self) -> &std::path::Path {
        &self.tls_ca_path
    }
}
