use tonic::transport::Server;

use server::DatabaseManager;
use storage_api::database_server::DatabaseServer;

pub mod facade;
pub mod server;
pub mod service;
pub mod storage_api;

mod storage_api_proto {
    include!("storage_api.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("database_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50054".parse()?; 
    let dbmanager = DatabaseManager::new();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(storage_api_proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    Server::builder()
        .add_service(DatabaseServer::new(dbmanager))
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    Ok(())
}
