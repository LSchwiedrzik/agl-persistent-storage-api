use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::storage_api::database_server::Database;
use crate::storage_api::{
    DestroyArguments, Key, KeyValue, ListResponse, ReadResponse, StandardResponse,
};

use crate::service::DbService;

#[derive(Debug)]
pub struct DatabaseManager {
    db_service: Arc<Mutex<DbService>>,
}

impl DatabaseManager {
    pub fn new() -> DatabaseManager {
        DatabaseManager {
            db_service: Arc::new(Mutex::new(DbService::new())),
        }
    }
}

#[tonic::async_trait]
impl Database for DatabaseManager {
    async fn destroy_db(
        &self,
        _request: Request<DestroyArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: (bool, String) = self.db_service.lock().await.destroy_db();

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn write(
        &self,
        request: Request<KeyValue>,
    ) -> Result<Response<StandardResponse>, Status> {
        let keyvalue = request.into_inner();
        let res: (bool, String) = self
            .db_service
            .lock()
            .await
            .write_db(&keyvalue.key, &keyvalue.value);

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn read(&self, request: Request<Key>) -> Result<Response<ReadResponse>, Status> {
        let key: Key = request.into_inner();
        let res: (bool, String, String) = self.db_service.lock().await.read_db(&key.key);

        Ok(Response::new(ReadResponse {
            success: res.0,
            message: res.1,
            result: res.2,
        }))
    }

    async fn delete(&self, request: Request<Key>) -> Result<Response<StandardResponse>, Status> {
        let key = request.into_inner();
        let res: (bool, String) = self.db_service.lock().await.delete_db(&key.key);

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn search(&self, request: Request<Key>) -> Result<Response<ListResponse>, Status> {
        let key: Key = request.into_inner();
        let res: (bool, String, Vec<String>) = self.db_service.lock().await.search_db(&key.key);

        Ok(Response::new(ListResponse {
            success: res.0,
            message: res.1,
            result: res.2,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_api::database_client::DatabaseClient;
    use crate::storage_api::database_server::DatabaseServer;
    use serial_test::serial;
    use std::net::SocketAddr;
    use tonic::transport::{Channel, Server};

    // TESTS FOR WRITE FUNCTION

    #[tokio::test]
    #[serial]
    async fn test_write_key_value() {
        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key = "Vehicle.Infotainment.Radio.CurrentStation";
        let value = "1live";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        };

        // Act
        let response = client.write(key_value).await.unwrap();
        let read_value = client
            .read(Key {
                key: key.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(response.into_inner().success && read_value.into_inner().result == value);

        // Clean up.
        let _response_destroy = client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_write_two_keys_on_root_level() {
        // Tests if it is possible to write to a "node" of a before hand written key, regarding VSS

        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key1 = "Vehicle";
        let value1 = "car";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
        };

        let key2 = "test";
        let value2 = "test";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
        };

        // Act
        let response1 = client.write(key_value1).await.unwrap();
        let response2 = client.write(key_value2).await.unwrap();

        let read_value1 = client
            .read(Key {
                key: key1.to_string(),
            })
            .await
            .unwrap();
        let read_value2 = client
            .read(Key {
                key: key2.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(
            read_value1.into_inner().result == value1
                && read_value2.into_inner().result == value2
                && response1.into_inner().success
                && response2.into_inner().success
        );

        // Clean up.
        let _response_destroy = client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_write_to_node() {
        // Tests if it is possible to write to a "node" of a before hand written key, regarding VSS

        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let value1 = "1live";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
        };

        let key2 = "Vehicle.Infotainment";
        let value2 = "exists";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
        };

        // Act
        let response1 = client.write(key_value1).await.unwrap();
        let response2 = client.write(key_value2).await.unwrap();

        let read_value1 = client
            .read(Key {
                key: key1.to_string(),
            })
            .await
            .unwrap();
        let read_value2 = client
            .read(Key {
                key: key2.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(
            read_value1.into_inner().result == value1
                && read_value2.into_inner().result == value2
                && response1.into_inner().success
                && response2.into_inner().success
        );

        // Clean up.
        let _response_destroy = client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    // TESTS FOR DELETE FUNCTION

    #[tokio::test]
    #[serial]
    async fn test_delete() {
        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key = "Vehicle.Infotainment.Radio.CurrentStation";
        let value = "1live";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
        };

        // Act
        let response_write = client.write(key_value).await.unwrap();
        let response_delete = client
            .delete(Key {
                key: key.to_string(),
            })
            .await
            .unwrap();
        let response_read = client
            .read(Key {
                key: key.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(
            response_write.into_inner().success
                && response_delete.into_inner().success
                && !response_read.into_inner().success
        );

        // Clean up.
        let _response_destroy = client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_key_does_not_exist() {
        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key = "Key.doesNotExist";

        // Act
        let response_read = client
            .read(Key {
                key: key.to_string(),
            })
            .await
            .unwrap();
        let response_delete = client
            .delete(Key {
                key: key.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(!response_delete.into_inner().success && !response_read.into_inner().success);

        // Clean up.
        let _response_destroy = client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    // TESTS FOR SEARCH FUNCTION

    async fn fill_db_for_search_tests(
        client: &mut DatabaseClient<Channel>,
        key1: &str,
        key2: &str,
    ) {
        let value1 = "1live";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
        };
        let response1 = client.write(key_value1).await.unwrap();
        assert!(response1.into_inner().success);

        let value2 = "10";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
        };
        let response2 = client.write(key_value2).await.unwrap();
        assert!(response2.into_inner().success);

        let key3 = "Vehicle.Infotainment.Display.Color";
        let value3 = "blue";
        let key_value3 = KeyValue {
            key: key3.to_string(),
            value: value3.to_string(),
        };
        let response3 = client.write(key_value3).await.unwrap();
        assert!(response3.into_inner().success);
    }

    #[tokio::test]
    #[serial]
    async fn test_full_search_key() {
        // list_keys_containing('Radio') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Communication.Radio.Volume')

        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "Radio";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(search_response.success);
        assert_eq!(search_response.result, vec![key2, key1]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_partial_search_key() {
        // list_keys_containing('Rad') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Communication.Radio.Volume')

        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "Rad";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(search_response.success);
        assert_eq!(search_response.result, vec![key2, key1]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_search_key_contains_dot() {
        // list_keys_containing('nt.Rad') -> ('Vehicle.Infotainment.Radio.CurrentStation')

        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "nt.Rad";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(search_response.success);
        assert_eq!(search_response.result, vec![key1]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_empty_search_key() {
        // list_keys_containing('') -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')

        // Arrange
        let address: SocketAddr = "127.0.0.1:9001".parse().unwrap();
        let database_manager = DatabaseManager::new();
        let server = Server::builder().add_service(DatabaseServer::new(database_manager));
        let server_task = tokio::spawn(server.serve(address.clone()));

        // Wait for the server to be ready.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let end_addr = "http://127.0.0.1:9001";
        let endpoint = tonic::transport::Endpoint::from_static(end_addr);
        let mut client = DatabaseClient::connect(endpoint).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(search_response.success);
        assert_eq!(search_response.result, vec![key2, "Vehicle.Infotainment.Display.Color", key1]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }
}
