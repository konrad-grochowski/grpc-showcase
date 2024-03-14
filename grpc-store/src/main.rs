use std::collections::HashMap;

use grpc_codegen::key_value_storage_server::KeyValueStorage;
use grpc_codegen::key_value_storage_server::KeyValueStorageServer;
use grpc_codegen::{LoadReply, LoadRequest, StoreRequest};
use tokio::sync::RwLock;
use tonic::transport::ServerTlsConfig;
use tonic::{
    transport::{Identity, Server},
    Request, Response, Status,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let certs_dir = std::path::PathBuf::from_iter(["/self-signed-certs/grpc-store"]);
    let cert = std::fs::read_to_string(certs_dir.join("cert.pem"))?;
    let key = std::fs::read_to_string(certs_dir.join("key.pem"))?;

    let identity = Identity::from_pem(cert, key);
    let addr = "[::]:3001".parse()?;
    let key_value_storage = InMemoryKeyValueStorage::new();

    tracing::info!("Starting gRPC server...");
    Server::builder()
        .tls_config(ServerTlsConfig::new().identity(identity))?
        .add_service(KeyValueStorageServer::new(key_value_storage))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct InMemoryKeyValueStorage {
    map: RwLock<HashMap<String, String>>,
}

impl InMemoryKeyValueStorage {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }
}

#[tonic::async_trait]
impl KeyValueStorage for InMemoryKeyValueStorage {
    async fn store_key_value(
        &self,
        request: Request<StoreRequest>,
    ) -> Result<Response<()>, Status> {
        tracing::info!(?request, "Got a store request");
        let mut map_guard = self.map.write().await;
        let inner = request.into_inner();
        map_guard.insert(inner.key, inner.value);
        tracing::info!("Successfully inserted key");
        Ok(Response::new(()))
    }

    async fn load_key_value(
        &self,
        request: Request<LoadRequest>,
    ) -> Result<Response<LoadReply>, Status> {
        let map_guard = self.map.read().await;

        let key = request.into_inner().key;
        let value = map_guard
            .get(&key)
            .ok_or_else(|| tonic::Status::not_found("There is no entry under provided key"))?
            .clone();

        let res = tonic::Response::new(LoadReply { key, value });
        tracing::info!(?res, "Sending response");

        Ok(res)
    }
}

#[tokio::test]
async fn store_in_memory() -> anyhow::Result<()> {
    let storage = InMemoryKeyValueStorage::new();
    let (key, value): (String, String) = ("key".into(), "value".into());

    let request = tonic::Request::new(StoreRequest {
        key: key.clone(),
        value: value.clone(),
    });
    let _response = storage.store_key_value(request).await?;
    assert_eq!(storage.map.read().await.get(&key), Some(&value));
    Ok(())
}

#[tokio::test]
async fn load_from_memory() -> anyhow::Result<()> {
    let storage = InMemoryKeyValueStorage::new();
    let (key, value): (String, String) = ("key".into(), "value".into());

    storage.map.write().await.insert(key.clone(), value.clone());
    let request = tonic::Request::new(LoadRequest { key: key.clone() });
    let response = storage.load_key_value(request).await?;
    assert_eq!(response.into_inner(), LoadReply { key, value });

    Ok(())
}
