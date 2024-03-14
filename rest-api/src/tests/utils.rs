use grpc_codegen::{
    key_value_storage_client::KeyValueStorageClient,
    key_value_storage_server::{KeyValueStorage, KeyValueStorageServer},
};

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::{Channel, Server};

/// Mock implementation for gRPC service.
/// asserts and returns key/value pair given in constructor
#[derive(Debug, Default)]
pub(super) struct MockKeyValueStorage {
    key: String,
    value: Option<String>,
}

impl MockKeyValueStorage {
    pub(super) fn new(key: String, value: impl Into<Option<String>>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

#[tonic::async_trait]
impl KeyValueStorage for MockKeyValueStorage {
    async fn store_key_value(
        &self,
        request: tonic::Request<grpc_codegen::StoreRequest>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let inner = request.into_inner();
        assert_eq!(inner.key, self.key);
        assert_eq!(Some(inner.value), self.value);
        Ok(tonic::Response::new(()))
    }

    async fn load_key_value(
        &self,
        request: tonic::Request<grpc_codegen::LoadRequest>,
    ) -> Result<tonic::Response<grpc_codegen::LoadReply>, tonic::Status> {
        assert_eq!(request.into_inner().key, self.key);

        let res = tonic::Response::new(grpc_codegen::LoadReply {
            key: self.key.clone(),
            value: self
                .value
                .clone()
                .ok_or_else(|| tonic::Status::not_found("There is no entry under provided key"))?,
        });

        Ok(res)
    }
}

/// Sets up a gRPC server on a free port.
/// Returns client to created server.
pub(super) async fn get_client_to_mock_grpc_server(
    mock_storage: MockKeyValueStorage,
) -> anyhow::Result<KeyValueStorageClient<Channel>> {
    // requesting binding to port 0, that makes OS return a free listener on a *free* port
    let tcp_listener = TcpListener::bind("0.0.0.0:0").await.unwrap();

    let grpc_addr: std::net::SocketAddr = tcp_listener.local_addr().unwrap();

    let stream = TcpListenerStream::new(tcp_listener);
    tracing::info!("Starting gRPC server...");
    tokio::task::spawn(
        Server::builder()
            .add_service(KeyValueStorageServer::new(mock_storage))
            .serve_with_incoming(stream),
    );
    let grpc_client: KeyValueStorageClient<Channel> =
        KeyValueStorageClient::connect(format!("http://{}", grpc_addr))
            .await
            .expect("Failed to connect to gRPC server");

    Ok(grpc_client)
}
