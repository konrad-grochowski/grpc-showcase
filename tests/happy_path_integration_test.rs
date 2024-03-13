// use testcontainers::core::WaitFor;
// use testcontainers::GenericImage;

// use testcontainers::RunnableImage;
// pub use testcontainers::{clients::Cli, Container};

// #[tokio::test]
// async fn happy_path() -> anyhow::Result<()> {
//     let network = "test-network";

//     let rest_api_image = GenericImage::new("rest-api", "latest")
//         .with_wait_for(WaitFor::message_on_stdout("Starting server..."));
//     let rest_api_image = RunnableImage::from(rest_api_image)
//         .with_network(network)
//         .with_container_name("rest-api-container");

//     let grpc_store_image = GenericImage::new("grpc_store", "latest")
//         .with_wait_for(WaitFor::message_on_stdout("Starting server..."));
//     let grpc_store_image = RunnableImage::from(grpc_store_image)
//         .with_network(network)
//         .with_container_name("grpc-store-container");

//     let docker = Cli::default();
//     // docker.exec("")
//     let _grpc_store_containe = docker.run(grpc_store_image);
//     let _rest_api_container = docker.run(rest_api_image);

//     Ok(())
// }

// // #[derive(Debug, Default)]
// // pub struct GrpcStore;

// // impl Image for GrpcStore {
// //     type Args = ();

// //     fn name(&self) -> String {
// //         "grpc-store".to_owned()
// //     }

// //     fn tag(&self) -> String {
// //         "latest".to_owned()
// //     }

// //     fn ready_conditions(&self) -> Vec<WaitFor> {
// //         vec![WaitFor::message_on_stdout("Starting gRPC server...")]
// //     }
// // }

// // #[derive(Debug, Default)]
// // pub struct RestApi;

// // impl Image for RestApi {
// //     type Args = ();

// //     fn name(&self) -> String {
// //         "rest-api".to_owned()
// //     }

// //     fn tag(&self) -> String {
// //         "latest".to_owned()
// //     }

// //     fn ready_conditions(&self) -> Vec<WaitFor> {
// //         vec![WaitFor::message_on_stdout("Starting server...")]
// //     }
// // }
