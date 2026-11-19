use anyhow::{anyhow, ensure, Result};
use rustls::pki_types::{pem::PemObject, CertificateDer};
use std::{io::Read, path::Path, sync::Arc, time::Duration};

pub(super) struct ConnectionConfig {
    url: String,
    credentials: Option<(String, String)>,
    database: Option<String>,
    ca_path: Option<String>,
}

impl ConnectionConfig {
    fn parse(values: [Option<String>; 5]) -> Result<Self> {
        let [url, user, password, database, ca_path] = values;
        let url = url.unwrap_or_else(|| "http://localhost:8123".into());
        let parsed = url::Url::parse(&url).map_err(|_| anyhow!("Invalid CLICKHOUSE_URL"))?;
        ensure!(
            matches!(parsed.scheme(), "http" | "https")
                && parsed.host_str().is_some()
                && parsed.username().is_empty()
                && parsed.password().is_none()
                && parsed.query().is_none()
                && parsed.fragment().is_none(),
            "CLICKHOUSE_URL must be an HTTP(S) endpoint without credentials, query or fragment"
        );
        let credentials = match (user, password) {
            (None, None) => None,
            (Some(user), Some(password))
                if !user.trim().is_empty()
                    && !password.is_empty()
                    && !user.chars().any(char::is_control)
                    && !password.chars().any(char::is_control) =>
            {
                Some((user, password))
            }
            _ => {
                return Err(anyhow!(
                "CLICKHOUSE_USER and CLICKHOUSE_PASSWORD must both be nonempty valid header values"
            ))
            }
        };
        ensure!(
            database
                .as_ref()
                .is_none_or(|v| !v.trim().is_empty() && !v.chars().any(char::is_control)),
            "Invalid CLICKHOUSE_DATABASE"
        );
        ensure!(
            ca_path.as_ref().is_none_or(|v| !v.is_empty()),
            "Invalid CLICKHOUSE_CA_CERT_PATH"
        );
        ensure!(
            (credentials.is_none() && ca_path.is_none()) || parsed.scheme() == "https",
            "ClickHouse credentials and custom CA require HTTPS"
        );
        Ok(Self {
            url,
            credentials,
            database,
            ca_path,
        })
    }
}

pub(super) fn from_env() -> Result<clickhouse::Client> {
    fn value(name: &str) -> Result<Option<String>> {
        match std::env::var(name) {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(_) => Err(anyhow!("{name} must be valid UTF-8")),
        }
    }
    connect(ConnectionConfig::parse([
        value("CLICKHOUSE_URL")?,
        value("CLICKHOUSE_USER")?,
        value("CLICKHOUSE_PASSWORD")?,
        value("CLICKHOUSE_DATABASE")?,
        value("CLICKHOUSE_CA_CERT_PATH")?,
    ])?)
}

fn trust_roots(ca_path: Option<&Path>) -> Result<rustls::RootCertStore> {
    let mut roots = rustls::RootCertStore::empty();
    if let Some(path) = ca_path {
        const MAX_CA_BYTES: u64 = 1024 * 1024;
        let file = std::fs::File::open(path)
            .map_err(|_| anyhow!("Cannot read CLICKHOUSE_CA_CERT_PATH"))?;
        let mut pem = Vec::new();
        file.take(MAX_CA_BYTES + 1)
            .read_to_end(&mut pem)
            .map_err(|_| anyhow!("Cannot read CLICKHOUSE_CA_CERT_PATH"))?;
        ensure!(
            pem.len() as u64 <= MAX_CA_BYTES,
            "ClickHouse CA bundle exceeds 1 MiB"
        );
        for cert in CertificateDer::pem_slice_iter(&pem) {
            let cert = cert.map_err(|_| anyhow!("Invalid ClickHouse CA PEM"))?;
            roots
                .add(cert)
                .map_err(|_| anyhow!("Invalid ClickHouse CA certificate"))?;
        }
        ensure!(
            !roots.is_empty(),
            "ClickHouse CA bundle contains no certificates"
        );
    } else {
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }
    Ok(roots)
}

pub(super) fn connect(config: ConnectionConfig) -> Result<clickhouse::Client> {
    let roots = trust_roots(config.ca_path.as_deref().map(Path::new))?;
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(|_| anyhow!("Cannot configure ClickHouse TLS"))?
    .with_root_certificates(roots)
    .with_no_client_auth();
    let mut tcp = hyper_util::client::legacy::connect::HttpConnector::new();
    tcp.enforce_http(false);
    tcp.set_keepalive(Some(Duration::from_secs(60)));
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .wrap_connector(tcp);
    let http = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .pool_idle_timeout(Duration::from_secs(2))
        .build(connector);
    // Hyper does not follow redirects or consult proxy environment variables.
    let mut client = clickhouse::Client::with_http_client(http).with_url(config.url);
    if let Some((user, password)) = config.credentials {
        client = client.with_user(user).with_password(password);
    }
    if let Some(database) = config.database {
        client = client.with_database(database);
    }
    Ok(client)
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
