///! HTTP server acting as a procy to a gRPC service
/// 
/// See README for more information.
#[cfg(test)]
mod tests;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::routing::post;
use axum::{extract::State, routing::get, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use grpc_codegen::key_value_storage_client::KeyValueStorageClient;
use grpc_codegen::StoreRequest;
use tonic::transport::Channel;

use grpc_codegen::{LoadReply, LoadRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("Getting gRPC connection...");

    // As per documentation, cloning the gRPC client is cheap.
    // This makes connection multiplexing with `Arc<Mutex<...>>` redundant.
    // https://docs.rs/tonic/latest/tonic/client/index.html
    let channel = tls_grpc_channel().await?;
    let grpc_client: KeyValueStorageClient<Channel> = KeyValueStorageClient::new(channel);

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(
        PathBuf::from("/self-signed-certs/rest-api").join("cert.pem"),
        PathBuf::from("/self-signed-certs/rest-api").join("key.pem"),
    )
    .await?;

    let app = app(grpc_client);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting server...");
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

fn app(grpc_client: KeyValueStorageClient<Channel>) -> Router {
    Router::new()
        .route("/store", post(store_key_value))
        .route("/load", get(load_key_value))
        .with_state(grpc_client)
}

async fn tls_grpc_channel() -> anyhow::Result<Channel> {
    let data_dir = std::path::PathBuf::from_iter(["/self-signed-certs"]);

    let ca = std::fs::read_to_string(data_dir.join("grpc-store/rootCA.crt")).unwrap();
    let ca = tonic::transport::Certificate::from_pem(ca);

    let tls = tonic::transport::ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("grpc-store");

    let channel = Channel::from_static("https://grpc-store:3001")
        .tls_config(tls)?
        .connect()
        .await?;
    Ok(channel)
}

/// Handles requests to store a key-value pair inside the gRPC memory.
async fn store_key_value(
    State(mut grpc_client): State<KeyValueStorageClient<Channel>>,
    Json(store_request): Json<StoreRequest>,
) -> hyper::StatusCode {
    tracing::info!(?store_request, "Received storing request");

    let tonic_request = tonic::Request::new(store_request);
    let reply_result: Result<tonic::Response<()>, tonic::Status> =
        grpc_client.store_key_value(tonic_request).await;

    let Err(status) = reply_result else {
        tracing::info!("Key storing endpoint returned StatusCode::OK");
        return hyper::StatusCode::OK;
    };

    let hyper_status_code = tonic_status_into_hyper_status_code(status);
    tracing::warn!(
        ?hyper_status_code,
        "Key storing endpoint returned **non-ok** status code"
    );
    hyper_status_code
}

/// Handles requests to load a key-value pair inside the gRPC memory, given the key.
/// Returns key-value pair in case of successful gRPC reply.
async fn load_key_value(
    State(mut grpc_client): State<KeyValueStorageClient<Channel>>,
    Json(load_request): Json<LoadRequest>,
) -> Result<Json<LoadReply>, hyper::StatusCode> {
    tracing::info!(?load_request, "Received load request");

    let tonic_request = tonic::Request::new(load_request);
    let reply_result: Result<tonic::Response<LoadReply>, tonic::Status> =
        grpc_client.load_key_value(tonic_request).await;

    let status: tonic::Status = match reply_result {
        Ok(tonic_response) => return Ok(Json(tonic_response.into_inner())),
        Err(status) => status,
    };

    let hyper_status_code = tonic_status_into_hyper_status_code(status);
    tracing::warn!(
        ?hyper_status_code,
        "Key loading endpoint returned **non-ok** status code"
    );
    Err(hyper_status_code)
}

/// Converts `tonic::Status` into `hyper::StatusCode`.
fn tonic_status_into_hyper_status_code(tonic_status: tonic::Status) -> hyper::StatusCode {
    match tonic_status.code() {
        tonic::Code::Ok => hyper::StatusCode::OK,
        tonic::Code::Cancelled => hyper::StatusCode::REQUEST_TIMEOUT,
        tonic::Code::Unknown => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        tonic::Code::InvalidArgument => hyper::StatusCode::BAD_REQUEST,
        tonic::Code::DeadlineExceeded => hyper::StatusCode::GATEWAY_TIMEOUT,
        tonic::Code::NotFound => hyper::StatusCode::NOT_FOUND,
        tonic::Code::AlreadyExists => hyper::StatusCode::CONFLICT,
        tonic::Code::PermissionDenied => hyper::StatusCode::FORBIDDEN,
        tonic::Code::ResourceExhausted => hyper::StatusCode::TOO_MANY_REQUESTS,
        tonic::Code::FailedPrecondition => hyper::StatusCode::PRECONDITION_FAILED,
        tonic::Code::Aborted => hyper::StatusCode::CONFLICT,
        tonic::Code::OutOfRange => hyper::StatusCode::BAD_REQUEST,
        tonic::Code::Unimplemented => hyper::StatusCode::NOT_IMPLEMENTED,
        tonic::Code::Internal => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        tonic::Code::Unavailable => hyper::StatusCode::SERVICE_UNAVAILABLE,
        tonic::Code::DataLoss => hyper::StatusCode::INTERNAL_SERVER_ERROR,
        tonic::Code::Unauthenticated => hyper::StatusCode::UNAUTHORIZED,
    }
}
