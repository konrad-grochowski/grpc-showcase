use std::collections::HashMap;

use grpc_codegen::key_value_storage_server::KeyValueStorage;
use grpc_codegen::key_value_storage_server::KeyValueStorageServer;
use grpc_codegen::{LoadReply, LoadRequest, StoreRequest};
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:50051".parse()?;
    let greeter = InMemoryKeyValueStorage::new();

    Server::builder()
        .add_service(KeyValueStorageServer::new(greeter))
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
        tracing::debug!(?request, "Got a store request");
        let mut map_guard = self.map.write().await;
        let inner = request.into_inner();
        map_guard.insert(inner.key, inner.value);
        tracing::debug!("Successfully inserted key");
        Ok(Response::new(()))
    }

    async fn load_key_value(
        &self,
        request: Request<LoadRequest>,
    ) -> Result<Response<LoadReply>, Status> {
        let map_guard = self.map.read().await;

        let key = request.into_inner().key;
        let maybe_value = map_guard.get(&key).cloned();

        let res = tonic::Response::new(LoadReply {
            key: key,
            value: maybe_value.unwrap(),
        }); //TODO handle None
        tracing::debug!(?res, "Sending response");

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
    assert_eq!(
        response.into_inner(),
        LoadReply {
            key: key,
            value: value
        }
    );

    Ok(())
}
