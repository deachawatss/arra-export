use std::{net::IpAddr, sync::Arc, time::Duration};

use reqwest::{Client, StatusCode, header, redirect::Policy};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use url::{Host, Url};

use crate::{
    config::{ConfigError, parse_oracle_url},
    export::{ExportArtifact, ExportRequest},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub name: String,
    pub row_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectionsResponse {
    pub collections: Vec<Collection>,
    #[serde(default)]
    pub formats: Vec<String>,
    #[serde(default)]
    pub graph: Option<GraphCapability>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphCapability {
    pub collection: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub ok: bool,
    pub status: String,
    #[serde(default)]
    pub collections: Vec<Collection>,
    #[serde(default)]
    pub total_rows: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteExportJob {
    pub job_id: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub progress: Option<u8>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Error)]
pub enum OracleClientError {
    #[error("invalid Oracle URL: {0}")]
    InvalidUrl(#[from] ConfigError),
    #[error("could not resolve Oracle host")]
    HostResolution,
    #[error("Oracle URL must resolve only to public addresses or loopback")]
    UnsafeAddress,
    #[error("unable to create HTTP client: {0}")]
    Client(#[source] reqwest::Error),
    #[error("Oracle request failed: {0}")]
    Request(#[source] reqwest::Error),
    #[error("Oracle returned HTTP {status}: {message}")]
    Upstream { status: StatusCode, message: String },
    #[error("Oracle returned an invalid response: {0}")]
    Response(#[source] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct OracleClient {
    client: Client,
    target: Arc<RwLock<Url>>,
}

impl OracleClient {
    pub fn new(target: Url) -> Result<Self, OracleClientError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .redirect(Policy::none())
            .build()
            .map_err(OracleClientError::Client)?;

        Ok(Self {
            client,
            target: Arc::new(RwLock::new(target)),
        })
    }

    pub async fn current_url(&self) -> Url {
        self.target.read().await.clone()
    }

    pub async fn test_connection(
        &self,
        requested_url: &str,
    ) -> Result<ConnectionStatus, OracleClientError> {
        let target = parse_oracle_url(requested_url)?;
        verify_user_target(&target).await?;
        let url = endpoint(&target, "api/v1/export/test-connection")?;
        let response = self
            .client
            .post(url)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(OracleClientError::Request)?;
        let mut status: ConnectionStatus = decode_json(response).await?;
        status.url = Some(target.to_string());

        if status.ok {
            *self.target.write().await = target;
        }

        Ok(status)
    }

    pub async fn collections(&self) -> Result<CollectionsResponse, OracleClientError> {
        let target = self.current_url().await;
        let url = endpoint(&target, "api/v1/export/app/collections")?;
        let response = self
            .client
            .get(url)
            .header(header::ACCEPT, "application/json")
            .send()
            .await
            .map_err(OracleClientError::Request)?;
        decode_json(response).await
    }

    pub async fn start_export(
        &self,
        request: &ExportRequest,
    ) -> Result<(RemoteExportJob, Url), OracleClientError> {
        let target = self.current_url().await;
        let url = endpoint(&target, "api/v1/export/app/run")?;
        let response = self
            .client
            .post(url)
            .header(header::ACCEPT, "application/json")
            .json(request)
            .send()
            .await
            .map_err(OracleClientError::Request)?;
        let job = decode_json(response).await?;
        Ok((job, target))
    }

    pub async fn download(
        &self,
        oracle_url: &Url,
        remote_job_id: &str,
        extension: &str,
    ) -> Result<ExportArtifact, OracleClientError> {
        let url = endpoint(
            oracle_url,
            &format!("api/v1/export/app/download/{remote_job_id}"),
        )?;
        let response = self
            .client
            .get(url)
            .header(
                header::ACCEPT,
                "application/json, text/csv, text/markdown, application/x-ndjson",
            )
            .send()
            .await
            .map_err(OracleClientError::Request)?;
        let response = ensure_success(response).await?;
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_owned();
        let filename = filename_from_headers(response.headers(), remote_job_id, extension);
        let bytes = response
            .bytes()
            .await
            .map_err(OracleClientError::Response)?
            .to_vec();

        Ok(ExportArtifact {
            bytes,
            content_type,
            filename,
        })
    }
}

async fn verify_user_target(target: &Url) -> Result<(), OracleClientError> {
    match target.host() {
        Some(Host::Ipv4(address)) => verify_ip(IpAddr::V4(address)),
        Some(Host::Ipv6(address)) => verify_ip(IpAddr::V6(address)),
        Some(Host::Domain("localhost")) => Ok(()),
        Some(Host::Domain(host)) => {
            let port = target
                .port_or_known_default()
                .ok_or(OracleClientError::HostResolution)?;
            let resolved = tokio::net::lookup_host((host, port))
                .await
                .map_err(|_| OracleClientError::HostResolution)?
                .map(|address| address.ip())
                .collect::<Vec<_>>();
            if resolved.is_empty()
                || resolved
                    .into_iter()
                    .any(|address| verify_ip(address).is_err())
            {
                return Err(OracleClientError::UnsafeAddress);
            }
            Ok(())
        }
        None => Err(OracleClientError::HostResolution),
    }
}

fn verify_ip(address: IpAddr) -> Result<(), OracleClientError> {
    if address.is_loopback() || is_public_ip(address) {
        Ok(())
    } else {
        Err(OracleClientError::UnsafeAddress)
    }
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            !address.is_private()
                && !address.is_loopback()
                && !address.is_link_local()
                && !address.is_broadcast()
                && !address.is_documentation()
                && !address.is_unspecified()
                && !address.is_multicast()
        }
        IpAddr::V6(address) => {
            let segments = address.segments();
            let is_documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;
            !address.is_loopback()
                && !address.is_unspecified()
                && !address.is_multicast()
                && !address.is_unicast_link_local()
                && !address.is_unique_local()
                && !is_documentation
        }
    }
}

fn endpoint(base: &Url, path: &str) -> Result<Url, OracleClientError> {
    base.join(path)
        .map_err(|_| OracleClientError::HostResolution)
}

async fn decode_json<T>(response: reqwest::Response) -> Result<T, OracleClientError>
where
    T: serde::de::DeserializeOwned,
{
    ensure_success(response)
        .await?
        .json()
        .await
        .map_err(OracleClientError::Response)
}

async fn ensure_success(
    response: reqwest::Response,
) -> Result<reqwest::Response, OracleClientError> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let message = response
        .text()
        .await
        .unwrap_or_else(|_| "Oracle did not provide an error response".to_owned());
    Err(OracleClientError::Upstream { status, message })
}

fn filename_from_headers(headers: &header::HeaderMap, job_id: &str, extension: &str) -> String {
    let value = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split("filename=").nth(1))
        .map(|filename| filename.trim_matches(['"', '\'', ' ']))
        .filter(|filename| !filename.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("oracle-export-{job_id}.{extension}"));

    let safe = value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
        .collect::<String>();
    if safe.is_empty() {
        format!("oracle-export-{job_id}.{extension}")
    } else {
        safe
    }
}
