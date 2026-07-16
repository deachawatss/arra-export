use std::{path::PathBuf, sync::Arc};

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{
    config::AppConfig,
    export::{ExportRequest, JobManager},
    oracle_client::{CollectionsResponse, ConnectionStatus, OracleClient, OracleClientError},
};

#[derive(Clone)]
pub struct AppState {
    oracle: OracleClient,
    jobs: JobManager,
}

impl AppState {
    pub fn new(oracle: OracleClient) -> Self {
        Self {
            oracle,
            jobs: JobManager::default(),
        }
    }

    pub fn from_config(config: &AppConfig) -> Result<Self, OracleClientError> {
        Ok(Self::new(OracleClient::new(config.oracle_url.clone())?))
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
enum AppError {
    BadRequest(String),
    NotFound(String),
    Upstream(OracleClientError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::BadRequest(error) => (StatusCode::BAD_REQUEST, error),
            Self::NotFound(error) => (StatusCode::NOT_FOUND, error),
            Self::Upstream(error) => (StatusCode::BAD_GATEWAY, error.to_string()),
        };
        (status, Json(ErrorResponse { error })).into_response()
    }
}

impl From<OracleClientError> for AppError {
    fn from(value: OracleClientError) -> Self {
        match value {
            OracleClientError::InvalidUrl(error) => Self::BadRequest(error.to_string()),
            OracleClientError::HostResolution => {
                Self::BadRequest("Oracle host could not be resolved".to_owned())
            }
            OracleClientError::UnsafeAddress => Self::BadRequest(
                "Oracle URL must resolve only to public addresses or loopback".to_owned(),
            ),
            error => Self::Upstream(error),
        }
    }
}

#[derive(Debug, Deserialize)]
struct TestConnectionRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn api_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/test-connection", post(test_connection))
        .route("/api/collections", get(collections))
        .route("/api/export", post(create_export))
        .route("/api/export/{id}/download", get(download))
        .route("/api/export/history", get(history))
        .with_state(Arc::new(state))
}

pub fn app_router(state: AppState, frontend_dist: PathBuf) -> Router {
    let index = frontend_dist.join("index.html");
    api_router(state)
        .fallback_service(ServeDir::new(frontend_dist).not_found_service(ServeFile::new(index)))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn test_connection(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TestConnectionRequest>,
) -> Result<Json<ConnectionStatus>, AppError> {
    if request.url.trim().is_empty() {
        return Err(AppError::BadRequest("url is required".to_owned()));
    }
    Ok(Json(state.oracle.test_connection(&request.url).await?))
}

async fn collections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CollectionsResponse>, AppError> {
    Ok(Json(state.oracle.collections().await?))
}

async fn create_export(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<crate::export::ExportJob>, AppError> {
    request
        .validate()
        .map_err(|error| AppError::BadRequest(error.to_owned()))?;
    let (remote, oracle_url) = state.oracle.start_export(&request).await?;
    Ok(Json(state.jobs.record(&request, remote, oracle_url)))
}

async fn download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let job = state
        .jobs
        .get(&id)
        .ok_or_else(|| AppError::NotFound(format!("export job not found: {id}")))?;
    let artifact = state
        .oracle
        .download(
            &job.oracle_url,
            &job.remote_job_id,
            job.view.format.extension(),
        )
        .await?;

    let content_type = HeaderValue::from_str(&artifact.content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let disposition =
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", artifact.filename))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment"));
    let mut response = Response::new(Body::from(artifact.bytes));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);
    response
        .headers_mut()
        .insert(header::CONTENT_DISPOSITION, disposition);
    Ok(response)
}

async fn history(State(state): State<Arc<AppState>>) -> Json<Vec<crate::export::ExportJob>> {
    Json(state.jobs.history())
}
