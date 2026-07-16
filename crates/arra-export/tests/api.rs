use std::net::SocketAddr;

use arra_export::{
    app::{AppState, api_router},
    config::parse_oracle_url,
    oracle_client::OracleClient,
};
use axum::{
    Json, Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
    routing::{get, post},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn oracle_fixture() -> String {
    let app = Router::new()
        .route(
            "/api/v1/export/test-connection",
            post(|| async {
                Json(json!({
                    "ok": true,
                    "status": "connected",
                    "collections": [{"name": "oracle_documents", "rowCount": 3}],
                    "totalRows": 3
                }))
            }),
        )
        .route(
            "/api/v1/export/app/collections",
            get(|| async {
                Json(json!({
                    "collections": [{"name": "oracle_documents", "rowCount": 3}],
                    "formats": ["json", "csv", "markdown", "jsonl"],
                    "graph": {"collection": "relationships"}
                }))
            }),
        )
        .route(
            "/api/v1/export/app/run",
            post(|| async {
                Json(json!({
                    "jobId": "job-42",
                    "status": "completed",
                    "progress": 100,
                    "filename": "oracle_documents.json",
                    "createdAt": "2026-07-16T00:00:00Z"
                }))
            }),
        )
        .route(
            "/api/v1/export/app/download/job-42",
            get(|| async {
                Response::builder()
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"oracle_documents.json\"",
                    )
                    .body(Body::from("{\"documents\":3}\n"))
                    .expect("fixture response is valid")
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("fixture listener binds");
    let address: SocketAddr = listener
        .local_addr()
        .expect("fixture listener has a local address");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("fixture server remains healthy");
    });
    format!("http://{address}")
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body is readable")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("response body is JSON")
}

#[tokio::test]
async fn tests_connection_then_exposes_the_oracle_collections() {
    let oracle_url = oracle_fixture().await;
    let client = OracleClient::new(
        parse_oracle_url("http://localhost:47778").expect("default URL is valid"),
    )
    .expect("HTTP client is created");
    let app = api_router(AppState::new(client));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/test-connection")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"url": oracle_url}).to_string()))
                .expect("test request is valid"),
        )
        .await
        .expect("API responds");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_json(response).await["totalRows"], 3);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/collections")
                .body(Body::empty())
                .expect("test request is valid"),
        )
        .await
        .expect("API responds");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response_json(response).await["collections"][0]["name"],
        "oracle_documents"
    );
}

#[tokio::test]
async fn starts_an_export_and_proxies_the_artifact_download() {
    let oracle_url = oracle_fixture().await;
    let client = OracleClient::new(parse_oracle_url(&oracle_url).expect("fixture URL is valid"))
        .expect("HTTP client is created");
    let app = api_router(AppState::new(client));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/export")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({"collection": "oracle_documents", "format": "json", "includeGraph": true}).to_string(),
                ))
                .expect("test request is valid"),
        )
        .await
        .expect("API responds");
    assert_eq!(response.status(), StatusCode::OK);
    let job = response_json(response).await;
    assert_eq!(job["id"], "job-42");
    assert_eq!(job["downloadUrl"], "/api/export/job-42/download");

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/export/job-42/download")
                .body(Body::empty())
                .expect("test request is valid"),
        )
        .await
        .expect("API responds");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("download body is readable")
        .to_bytes();
    assert_eq!(bytes.as_ref(), b"{\"documents\":3}\n");
}

#[tokio::test]
async fn rejects_a_private_network_target_before_contacting_it() {
    let client = OracleClient::new(
        parse_oracle_url("http://localhost:47778").expect("default URL is valid"),
    )
    .expect("HTTP client is created");
    let app = api_router(AppState::new(client));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/test-connection")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"url":"http://10.0.0.1"}"#))
                .expect("test request is valid"),
        )
        .await
        .expect("API responds");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response_json(response).await["error"],
        "Oracle URL must resolve only to public addresses or loopback"
    );
}
