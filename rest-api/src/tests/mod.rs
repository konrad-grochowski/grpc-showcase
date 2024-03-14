mod utils;

use crate::tests::utils::{get_client_to_mock_grpc_server, MockKeyValueStorage};

use super::*;
use grpc_codegen::key_value_storage_server::{KeyValueStorage, KeyValueStorageServer};
use hyper::{body::Body, Request};
use tokio::{
    net::{tcp, TcpListener},
    task::JoinHandle,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tower::util::ServiceExt;

/// Verifies `/store` endpoint happy path.

#[tokio::test]
async fn store_key_value_happy_path() -> anyhow::Result<()> {
    let mock_storage = MockKeyValueStorage::new("key".into(), Some("value".into()));
    let grpc_client = get_client_to_mock_grpc_server(mock_storage).await?;

    let res = app(grpc_client)
        .oneshot(
            Request::builder()
                .method(hyper::Method::POST)
                .uri("/store")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(r#"{"key":"key", "value":"value"}"#.to_string())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), hyper::StatusCode::OK);
    Ok(())
}

/// Verifies `/load` endpoint happy path.
#[tokio::test]
async fn load_key_value_happy_path() -> anyhow::Result<()> {
    let mock_storage = MockKeyValueStorage::new("key".into(), Some("value".into()));
    let grpc_client = get_client_to_mock_grpc_server(mock_storage).await?;

    let res: hyper::Response<axum::body::Body> = app(grpc_client)
        .oneshot(
            Request::builder()
                .method(hyper::Method::GET)
                .uri("/load")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(r#"{"key":"key"}"#.to_string())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), hyper::StatusCode::OK);

    // check whether response body content is correct
    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await?;
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();
    assert_eq!(body_string, r#"{"key":"key","value":"value"}"#);
    Ok(())
}

/// Verifies `/load` endpoint happy path.
#[tokio::test]
async fn load_key_value_incorrect_payload() -> anyhow::Result<()> {
    let mock_storage = MockKeyValueStorage::new("key".into(), Some("value".into()));
    let grpc_client = get_client_to_mock_grpc_server(mock_storage).await?;

    let res: hyper::Response<axum::body::Body> = app(grpc_client)
        .oneshot(
            Request::builder()
                .method(hyper::Method::GET)
                .uri("/load")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(r#"INCORRECT_PAYLOAD"#.to_string())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), hyper::StatusCode::BAD_REQUEST);

    Ok(())
}



/// Verifies `/load` endpoint response to invalid payload.
#[tokio::test]
async fn store_key_value_incorrect_payload() -> anyhow::Result<()> {
    let mock_storage = MockKeyValueStorage::new("key".into(), Some("value".into()));
    let grpc_client = get_client_to_mock_grpc_server(mock_storage).await?;

    let res = app(grpc_client)
        .oneshot(
            Request::builder()
                .method(hyper::Method::POST)
                .uri("/store")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(r#"INCORRECT_PAYLOAD"#.to_string())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), hyper::StatusCode::BAD_REQUEST);
    Ok(())
}

/// Verifies `/load` endpoint response to invalid payload.
#[tokio::test]
async fn load_key_value_missing_key() -> anyhow::Result<()> {
    let mock_storage = MockKeyValueStorage::new("missing_key".into(), None);
    let grpc_client = get_client_to_mock_grpc_server(mock_storage).await?;

    let res: hyper::Response<axum::body::Body> = app(grpc_client)
        .oneshot(
            Request::builder()
                .method(hyper::Method::GET)
                .uri("/load")
                .header(
                    axum::http::header::CONTENT_TYPE,
                    mime::APPLICATION_JSON.as_ref(),
                )
                .body(r#"{"key": "missing_key"}"#.to_string())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), hyper::StatusCode::NOT_FOUND);


    Ok(())
}