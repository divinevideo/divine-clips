use super::*;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Server {
    url: String,
    ca: String,
    requests: Arc<Mutex<Vec<String>>>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn tls_server(name: &str) -> Server {
    use tokio_rustls::rustls::{self, pki_types::PrivatePkcs8KeyDer};
    let certificate = rcgen::generate_simple_self_signed(vec![name.into()]).unwrap();
    let ca = certificate.cert.pem();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(
        vec![certificate.cert.der().clone()],
        PrivatePkcs8KeyDer::from(certificate.key_pair.serialize_der()).into(),
    )
    .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "https://localhost:{}",
        listener.local_addr().unwrap().port()
    );
    let requests = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&requests);
    let task = tokio::spawn(async move {
        let (socket, _) = listener.accept().await.unwrap();
        let Ok(mut socket) = acceptor.accept(socket).await else {
            return;
        };
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") && request.len() < 16384 {
            let mut byte = [0];
            if socket.read_exact(&mut byte).await.is_err() {
                return;
            }
            request.push(byte[0]);
        }
        capture
            .lock()
            .unwrap()
            .push(String::from_utf8(request).unwrap());
        // One UInt8 in ClickHouse RowBinary format, with compression disabled by the test.
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n\x01")
            .await
            .unwrap();
        let _ = socket.shutdown().await;
    });
    Server {
        url,
        ca,
        requests,
        task,
    }
}

#[tokio::test]
async fn configured_private_tls_transmits_credentials_and_database() {
    let server = tls_server("localhost").await;
    let dir = tempfile::tempdir().unwrap();
    let ca = dir.path().join("ca.pem");
    std::fs::write(&ca, &server.ca).unwrap();
    let config = ConnectionConfig::parse([
        Some(server.url.clone()),
        Some("synthetic-reader".into()),
        Some("synthetic-password".into()),
        Some("analytics_fixture".into()),
        Some(ca.to_str().unwrap().into()),
    ])
    .unwrap();
    let client = connect(config)
        .unwrap()
        .with_compression(clickhouse::Compression::None);
    let value = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.query("SELECT 1").fetch_one::<u8>(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(value, 1);
    let requests = server.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let request = requests[0].to_ascii_lowercase();
    assert!(request.contains("x-clickhouse-user: synthetic-reader\r\n"));
    assert!(request.contains("x-clickhouse-key: synthetic-password\r\n"));
    assert!(request.contains("database=analytics_fixture"));
}

#[test]
fn malformed_configuration_never_echoes_input() {
    for values in [
        [
            Some("https://user:synthetic-secret@example.invalid"),
            None,
            None,
            None,
            None,
        ],
        [
            Some("https://example.invalid?password=synthetic-secret"),
            None,
            None,
            None,
            None,
        ],
        [Some("file:///synthetic-secret"), None, None, None, None],
        [None, Some("synthetic-secret"), None, None, None],
        [None, None, Some("synthetic-secret"), None, None],
        [None, Some(""), Some("synthetic-secret"), None, None],
        [None, None, None, Some(""), None],
        [None, None, None, None, Some("")],
    ] {
        let error = ConnectionConfig::parse(values.map(|v| v.map(str::to_owned)))
            .err()
            .expect("invalid config");
        assert!(!error.to_string().contains("synthetic-secret"));
    }
}

#[tokio::test]
async fn certificate_trust_and_hostname_are_both_required() {
    for case in ["public_roots", "wrong_ca", "wrong_hostname"] {
        let name = if case == "wrong_hostname" {
            "elsewhere.invalid"
        } else {
            "localhost"
        };
        let server = tls_server(name).await;
        let dir = tempfile::tempdir().unwrap();
        let ca = dir.path().join("ca.pem");
        let other = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let pem = if case == "wrong_ca" {
            other.cert.pem()
        } else {
            server.ca.clone()
        };
        std::fs::write(&ca, pem).unwrap();
        let config = ConnectionConfig::parse([
            Some(server.url.clone()),
            Some("synthetic-reader".into()),
            Some("synthetic-password".into()),
            None,
            (case != "public_roots").then(|| ca.to_str().unwrap().to_owned()),
        ])
        .unwrap();
        let client = connect(config)
            .unwrap()
            .with_compression(clickhouse::Compression::None);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            client.query("SELECT 1").fetch_one::<u8>(),
        )
        .await
        .unwrap();
        assert!(result.is_err(), "{case} must fail");
        assert!(
            server.requests.lock().unwrap().is_empty(),
            "credentials must not cross an unverified TLS connection"
        );
    }
}

#[test]
fn explicitly_configured_ca_cannot_fall_back_to_public_roots() {
    let dir = tempfile::tempdir().unwrap();
    let ca = dir.path().join("ca.pem");
    for bytes in [
        None,
        Some(Vec::new()),
        Some(b"not a certificate".to_vec()),
        Some(vec![b'x'; 1024 * 1024 + 1]),
    ] {
        if let Some(bytes) = bytes {
            std::fs::write(&ca, bytes).unwrap();
        }
        let config = ConnectionConfig::parse([
            Some("https://localhost:8443".into()),
            None,
            None,
            None,
            Some(ca.to_str().unwrap().into()),
        ])
        .unwrap();
        assert!(connect(config).is_err());
    }
}

#[tokio::test]
async fn anonymous_http_remains_available_for_local_development() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            socket.read_exact(&mut byte).await.unwrap();
            request.push(byte[0]);
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n\x01")
            .await
            .unwrap();
        String::from_utf8(request).unwrap()
    });
    let config = ConnectionConfig::parse([Some(url), None, None, None, None]).unwrap();
    let client = connect(config)
        .unwrap()
        .with_compression(clickhouse::Compression::None);
    let value = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        client.query("SELECT 1").fetch_one::<u8>(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(value, 1);
    let request = task.await.unwrap().to_ascii_lowercase();
    assert!(!request.contains("x-clickhouse-user:"));
    assert!(!request.contains("x-clickhouse-key:"));
}

#[test]
fn credentials_and_private_ca_never_allow_plaintext() {
    for values in [
        [
            Some("http://localhost:8123"),
            Some("synthetic-reader"),
            Some("synthetic-password"),
            None,
            None,
        ],
        [
            Some("http://localhost:8123"),
            None,
            None,
            None,
            Some("ca.pem"),
        ],
    ] {
        assert!(ConnectionConfig::parse(values.map(|v| v.map(str::to_owned))).is_err());
    }
    let config = ConnectionConfig::parse([None, None, None, None, None]).unwrap();
    assert_eq!(config.url, "http://localhost:8123");
    assert!(config.credentials.is_none());
    assert!(config.database.is_none());
}
