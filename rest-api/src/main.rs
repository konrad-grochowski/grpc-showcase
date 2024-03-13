use std::net::SocketAddr;

use axum::routing::post;
use axum::{extract::State, routing::get, Json, Router};
use grpc_codegen::key_value_storage_client::KeyValueStorageClient;
use grpc_codegen::StoreRequest;
use tonic::transport::Channel;

use grpc_codegen::{LoadReply, LoadRequest};




#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Getting gRPC connection...");

    // As per documentation, cloning the gRPC client is cheap.
    // This makes connection multiplexing with `Arc<Mutex<...>>` redundant.
    // https://docs.rs/tonic/latest/tonic/client/index.html
    let grpc_client: KeyValueStorageClient<Channel> =
        KeyValueStorageClient::connect("http://[::1]:50051")
            .await
            .expect("Failed to connect to gRPC server");

    let app = Router::new()
        .route("/store", post(store_key_value))
        .route("/load", get(load_key_value))
        .with_state(grpc_client);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting server...");
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to set up HTTP server");
}

/// Handles requests to store a key-value pair inside the gRPC memory.
async fn store_key_value(
    State(mut grpc_client): State<KeyValueStorageClient<Channel>>,
    Json(store_request): Json<StoreRequest>,
) -> hyper::StatusCode {
    tracing::debug!(?store_request, "Received storing request");

    let tonic_request = tonic::Request::new(store_request);
    let reply_result: Result<tonic::Response<()>, tonic::Status> =
        grpc_client.store_key_value(tonic_request).await;

    let Err(status) = reply_result else {
        tracing::debug!("Key storing endpoint returned StatusCode::OK");
        return hyper::StatusCode::OK;
    };
    // TODO: delegate into separate impl block (newtype pattern?)
    let hyper_status_code = tonic_status_into_hyper_status_code(status);
    tracing::warn!(
        ?hyper_status_code,
        "Key storing endpoint returned **non-ok** status code"
    );
    hyper_status_code
}


/// Handles requests to load a key-value pair inside the gRPC memory, given the key.
async fn load_key_value(
    State(mut grpc_client): State<KeyValueStorageClient<Channel>>,
    Json(load_request): Json<LoadRequest>,
) -> Result<Json<LoadReply>, hyper::StatusCode> {
    tracing::debug!(?load_request, "Received load request");

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
