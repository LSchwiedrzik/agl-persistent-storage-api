use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::storage_api::database_server::Database;
use crate::storage_api::{
    DestroyArguments, Key, KeyValue, ListResponse, ReadResponse, StandardResponse, SubtreeInfo,
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
        let res: (bool, String) = self.db_service.lock().await.write_db(
            &keyvalue.key,
            &keyvalue.value,
            &keyvalue.namespace,
        );

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn read(&self, request: Request<Key>) -> Result<Response<ReadResponse>, Status> {
        let key: Key = request.into_inner();
        let res: (bool, String, String) = self
            .db_service
            .lock()
            .await
            .read_db(&key.key, &key.namespace);

        Ok(Response::new(ReadResponse {
            success: res.0,
            message: res.1,
            result: res.2,
        }))
    }

    async fn delete(&self, request: Request<Key>) -> Result<Response<StandardResponse>, Status> {
        let key = request.into_inner();
        let res: (bool, String) = self
            .db_service
            .lock()
            .await
            .delete_db(&key.key, &key.namespace);

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn search(&self, request: Request<Key>) -> Result<Response<ListResponse>, Status> {
        let key: Key = request.into_inner();
        let res: (bool, String, Vec<String>) = self
            .db_service
            .lock()
            .await
            .search_db(&key.key, &key.namespace);

        Ok(Response::new(ListResponse {
            success: res.0,
            message: res.1,
            result: res.2,
        }))
    }

    async fn delete_recursively_from(
        &self,
        request: Request<Key>,
    ) -> Result<Response<StandardResponse>, Status> {
        let key: Key = request.into_inner();
        let res: (bool, String) = self
            .db_service
            .lock()
            .await
            .delete_recursively_from_db(&key.key, &key.namespace);

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn nodes_starting_in(
        &self,
        request: Request<SubtreeInfo>,
    ) -> Result<Response<ListResponse>, Status> {
        let stinfo: SubtreeInfo = request.into_inner();
        let res: (bool, String, Vec<String>) = self.db_service.lock().await.nodes_starting_in(
            &stinfo.node,
            stinfo.layers,
            &stinfo.namespace,
        );

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
    use rocksdb::statistics::NameParseError;
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key = "Vehicle.Infotainment.Radio.CurrentStation";
        let value = "1live";
        let namespace = "";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            namespace: namespace.to_string(),
        };

        // Act
        let response = client.write(key_value).await.unwrap();
        let read_value = client
            .read(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
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
    async fn test_write_empty_key() {
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key = "";
        let value = "test";
        let namespace = "";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            namespace: namespace.to_string(),
        };

        // Act
        let response_write = client.write(key_value).await.unwrap();
        let response_read = client
            .read(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(!response_write.into_inner().success);
        assert!(!response_read.into_inner().success);

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key1 = "Vehicle";
        let value1 = "car";
        let namespace1 = "";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
            namespace: namespace1.to_string(),
        };

        let key2 = "test";
        let value2 = "test";
        let namespace2 = "";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
            namespace: namespace2.to_string(),
        };

        // Act
        let response1 = client.write(key_value1).await.unwrap();
        let response2 = client.write(key_value2).await.unwrap();

        let read_value1 = client
            .read(Key {
                key: key1.to_string(),
                namespace: namespace1.to_string(),
            })
            .await
            .unwrap();
        let read_value2 = client
            .read(Key {
                key: key2.to_string(),
                namespace: namespace2.to_string(),
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let value1 = "1live";
        let namespace1 = "";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
            namespace: namespace1.to_string(),
        };

        let key2 = "Vehicle.Infotainment";
        let value2 = "exists";
        let namespace2 = "";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
            namespace: namespace2.to_string(),
        };

        // Act
        let response1 = client.write(key_value1).await.unwrap();
        let response2 = client.write(key_value2).await.unwrap();

        let read_value1 = client
            .read(Key {
                key: key1.to_string(),
                namespace: namespace1.to_string(),
            })
            .await
            .unwrap();
        let read_value2 = client
            .read(Key {
                key: key2.to_string(),
                namespace: namespace2.to_string(),
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
    async fn test_write_to_nondefault_namespace() {
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key = "Private.Info";
        let value = "test";
        let namespace = "AppName";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            namespace: namespace.to_string(),
        };

        // Act
        let response = client.write(key_value).await.unwrap();
        let read_value = client
            .read(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(response.into_inner().success && read_value.into_inner().result == value);

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key = "Vehicle.Infotainment.Radio.CurrentStation";
        let value = "1live";
        let namespace = "";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            namespace: namespace.to_string(),
        };

        // Act
        let response_write = client.write(key_value).await.unwrap();
        let response_delete = client
            .delete(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap();
        let response_read = client
            .read(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key = "Key.doesNotExist";
        let namespace = "";

        // Act
        let response_read = client
            .read(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap();
        let response_delete = client
            .delete(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(!response_delete.into_inner().success && !response_read.into_inner().success);

        // Clean up.
        let _response_destroy = client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_from_nondefault_namespace() {
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key = "Private.Info";
        let value = "test";
        let namespace = "AppName";
        let key_value = KeyValue {
            key: key.to_string(),
            value: value.to_string(),
            namespace: namespace.to_string(),
        };

        // Act
        let response_write = client.write(key_value).await.unwrap();
        let response_delete = client
            .delete(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap();
        let response_read = client
            .read(Key {
                key: key.to_string(),
                namespace: namespace.to_string(),
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

    // TESTS FOR SEARCH FUNCTION

    async fn fill_db_for_search_tests(
        client: &mut DatabaseClient<Channel>,
        key1: &str,
        key2: &str,
    ) {
        let value1 = "1live";
        let namespace1 = "";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
            namespace: namespace1.to_string(),
        };
        let response1 = client.write(key_value1).await.unwrap();
        assert!(response1.into_inner().success);

        let value2 = "10";
        let namespace2 = "";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
            namespace: namespace2.to_string(),
        };
        let response2 = client.write(key_value2).await.unwrap();
        assert!(response2.into_inner().success);

        let key3 = "Vehicle.Infotainment.Display.Color";
        let value3 = "blue";
        let namespace3 = "";
        let key_value3 = KeyValue {
            key: key3.to_string(),
            value: value3.to_string(),
            namespace: namespace3.to_string(),
        };
        let response3 = client.write(key_value3).await.unwrap();
        assert!(response3.into_inner().success);

        let key4 = "Private.Info";
        let value4 = "test";
        let namespace4 = "AppName";
        let key_value4 = KeyValue {
            key: key4.to_string(),
            value: value4.to_string(),
            namespace: namespace4.to_string(),
        };
        let response4 = client.write(key_value4).await.unwrap();
        assert!(response4.into_inner().success);
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "Radio";
        let namespace = "";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
                namespace: namespace.to_string(),
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "Rad";
        let namespace = "";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
                namespace: namespace.to_string(),
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "nt.Rad";
        let namespace = "";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
                namespace: namespace.to_string(),
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        let key1 = "Vehicle.Infotainment.Radio.CurrentStation";
        let key2 = "Vehicle.Communication.Radio.Volume";
        fill_db_for_search_tests(&mut client, key1, key2).await;

        // Act
        let searchstring = "";
        let namespace = "";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(search_response.success);
        assert_eq!(
            search_response.result,
            vec![key2, "Vehicle.Infotainment.Display.Color", key1]
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_partial_search_key_in_nondefault_namespace() {
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
        let searchstring = "Info";
        let namespace = "AppName";
        let search_response = client
            .search(Key {
                key: searchstring.to_string(),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(search_response.success);
        assert_eq!(search_response.result, vec!["Private.Info"]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    // TESTS FOR DELETE RECURSIVELY FROM

    #[tokio::test]
    #[serial]
    async fn test_delete_recursively() {
        // delete_recursively_from('Vehicle.Infotainment')
        // -> deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        // fill db
        let key1 = "Vehicle.Infotainment";
        let value1 = "test";
        let namespace1 = "";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
            namespace: namespace1.to_string(),
        };

        let response1 = client.write(key_value1).await.unwrap();
        assert!(response1.into_inner().success);

        let key2 = "Vehicle.Infotainment.Radio.CurrentStation";
        let value2 = "WDR 4";
        let namespace2 = "";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
            namespace: namespace2.to_string(),
        };

        let response2 = client.write(key_value2).await.unwrap();
        assert!(response2.into_inner().success);

        let key3 = "Vehicle.Infotainment.Radio.Volume";
        let value3 = "99%";
        let namespace3 = "";
        let key_value3 = KeyValue {
            key: key3.to_string(),
            value: value3.to_string(),
            namespace: namespace3.to_string(),
        };

        let response3 = client.write(key_value3).await.unwrap();
        assert!(response3.into_inner().success);

        let key4 = "Vehicle.Infotainment.HVAC.OutdoorTemperature";
        let value4 = "34 °C";
        let namespace4 = "";
        let key_value4 = KeyValue {
            key: key4.to_string(),
            value: value4.to_string(),
            namespace: namespace4.to_string(),
        };

        let response4 = client.write(key_value4).await.unwrap();
        assert!(response4.into_inner().success);

        let key5 = "Private.Info";
        let value5 = "test";
        let namespace5 = "AppName";
        let key_value5 = KeyValue {
            key: key5.to_string(),
            value: value5.to_string(),
            namespace: namespace5.to_string(),
        };
        let response5 = client.write(key_value5).await.unwrap();
        assert!(response5.into_inner().success);

        // Act
        let deletion_node = "Vehicle.Infotainment";
        let deletion_namespace = "";
        let delete_recursively_response = client
            .delete_recursively_from(Key {
                key: deletion_node.to_string(),
                namespace: deletion_namespace.to_string(),
            })
            .await
            .unwrap();

        let read_response1 = client
            .read(Key {
                key: key1.to_string(),
                namespace: namespace1.to_string(),
            })
            .await
            .unwrap();
        let read_response2 = client
            .read(Key {
                key: key2.to_string(),
                namespace: namespace2.to_string(),
            })
            .await
            .unwrap();
        let read_response3 = client
            .read(Key {
                key: key3.to_string(),
                namespace: namespace3.to_string(),
            })
            .await
            .unwrap();
        let read_response4 = client
            .read(Key {
                key: key4.to_string(),
                namespace: namespace4.to_string(),
            })
            .await
            .unwrap();
        let read_response5 = client
            .read(Key {
                key: key5.to_string(),
                namespace: namespace5.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(
            delete_recursively_response.into_inner().success
                && !read_response1.into_inner().success
                && !read_response2.into_inner().success
                && !read_response3.into_inner().success
                && !read_response4.into_inner().success
                && read_response5.into_inner().success
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_recursively_vehicle() {
        // delete_recursively_from('Vehicle') ->
        // deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        // fill db
        let key1 = "Vehicle.Infotainment";
        let value1 = "test";
        let namespace1 = "";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
            namespace: namespace1.to_string(),
        };
        let response1 = client.write(key_value1).await.unwrap();
        assert!(response1.into_inner().success);

        let key2 = "Vehicle.Infotainment.Radio.CurrentStation";
        let value2 = "WDR 4";
        let namespace2 = "";

        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
            namespace: namespace2.to_string(),
        };

        let response2 = client.write(key_value2).await.unwrap();
        assert!(response2.into_inner().success);

        let key3 = "Vehicle.Infotainment.Radio.Volume";
        let value3 = "99%";
        let namespace3 = "";
        let key_value3 = KeyValue {
            key: key3.to_string(),
            value: value3.to_string(),
            namespace: namespace3.to_string(),
        };
        let response3 = client.write(key_value3).await.unwrap();
        assert!(response3.into_inner().success);

        let key4 = "Vehicle.Infotainment.HVAC.OutdoorTemperature";
        let value4 = "34 °C";
        let namespace4 = "";
        let key_value4 = KeyValue {
            key: key4.to_string(),
            value: value4.to_string(),
            namespace: namespace4.to_string(),
        };
        let response4 = client.write(key_value4).await.unwrap();
        assert!(response4.into_inner().success);

        let key5 = "Vehicle.Communication.Radio.Volume";
        let value5 = "80%";
        let namespace5 = "";
        let key_value5 = KeyValue {
            key: key5.to_string(),
            value: value5.to_string(),
            namespace: namespace5.to_string(),
        };
        let response5 = client.write(key_value5).await.unwrap();
        assert!(response5.into_inner().success);

        let key6 = "Private.Info";
        let value6 = "test";
        let namespace6 = "AppName";
        let key_value6 = KeyValue {
            key: key6.to_string(),
            value: value6.to_string(),
            namespace: namespace6.to_string(),
        };
        let response6 = client.write(key_value6).await.unwrap();
        assert!(response6.into_inner().success);

        // Act
        let deletion_node = "Vehicle";
        let deletion_namespace = "";
        let delete_recursively_response = client
            .delete_recursively_from(Key {
                key: deletion_node.to_string(),
                namespace: deletion_namespace.to_string(),
            })
            .await
            .unwrap();

        let read_response1 = client
            .read(Key {
                key: key1.to_string(),
                namespace: namespace1.to_string(),
            })
            .await
            .unwrap();
        let read_response2 = client
            .read(Key {
                key: key2.to_string(),
                namespace: namespace2.to_string(),
            })
            .await
            .unwrap();
        let read_response3 = client
            .read(Key {
                key: key3.to_string(),
                namespace: namespace3.to_string(),
            })
            .await
            .unwrap();
        let read_response4 = client
            .read(Key {
                key: key4.to_string(),
                namespace: namespace4.to_string(),
            })
            .await
            .unwrap();
        let read_response5 = client
            .read(Key {
                key: key5.to_string(),
                namespace: namespace5.to_string(),
            })
            .await
            .unwrap();
        let read_response6 = client
            .read(Key {
                key: key6.to_string(),
                namespace: namespace6.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(
            delete_recursively_response.into_inner().success
                && !read_response1.into_inner().success
                && !read_response2.into_inner().success
                && !read_response3.into_inner().success
                && !read_response4.into_inner().success
                && !read_response5.into_inner().success
                && read_response6.into_inner().success
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_recursively_empty_key() {
        // delete_recursively_from('Vehicle') ->
        // deletes ('Vehicle.Infotainment', 'Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        // fill db
        let key1 = "Vehicle.Infotainment";
        let value1 = "test";
        let namespace1 = "";
        let key_value1 = KeyValue {
            key: key1.to_string(),
            value: value1.to_string(),
            namespace: namespace1.to_string(),
        };

        let response1 = client.write(key_value1).await.unwrap();
        assert!(response1.into_inner().success);

        let key2 = "Vehicle.Infotainment.Radio.CurrentStation";
        let value2 = "WDR 4";
        let namespace2 = "";
        let key_value2 = KeyValue {
            key: key2.to_string(),
            value: value2.to_string(),
            namespace: namespace2.to_string(),
        };

        let response2 = client.write(key_value2).await.unwrap();
        assert!(response2.into_inner().success);

        let key3 = "Private.Info";
        let value3 = "test";
        let namespace3 = "AppName";
        let key_value3 = KeyValue {
            key: key3.to_string(),
            value: value3.to_string(),
            namespace: namespace3.to_string(),
        };
        let response3 = client.write(key_value3).await.unwrap();
        assert!(response3.into_inner().success);

        // Act
        let deletion_node = "";
        let deletion_namespace = "";
        let delete_recursively_response = client
            .delete_recursively_from(Key {
                key: deletion_node.to_string(),
                namespace: deletion_namespace.to_string(),
            })
            .await
            .unwrap();

        let read_response1 = client
            .read(Key {
                key: key1.to_string(),
                namespace: namespace1.to_string(),
            })
            .await
            .unwrap();
        let read_response2 = client
            .read(Key {
                key: key2.to_string(),
                namespace: namespace2.to_string(),
            })
            .await
            .unwrap();
        let read_response3 = client
            .read(Key {
                key: key3.to_string(),
                namespace: namespace3.to_string(),
            })
            .await
            .unwrap();

        // Assert
        assert!(
            !delete_recursively_response.into_inner().success
                && read_response1.into_inner().success
                && read_response2.into_inner().success
                && read_response3.into_inner().success
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    // TESTS FOR nodes_starting_in

    async fn fill_db_example_tree(client: &mut DatabaseClient<Channel>) {
        let kv1 = KeyValue {
            key: "Vehicle.Infotainment".to_string(),
            value: "AGL_Infotainment".to_string(),
            namespace: "".to_string(),
        };
        let response1 = client.write(kv1).await.unwrap();
        assert!(response1.into_inner().success);

        let kv2 = KeyValue {
            key: "Vehicle.Infotainment.Radio.CurrentStation".to_string(),
            value: "1live".to_string(),
            namespace: "".to_string(),
        };
        let response2 = client.write(kv2).await.unwrap();
        assert!(response2.into_inner().success);

        let kv3 = KeyValue {
            key: "Vehicle.Infotainment.Radio.Volume".to_string(),
            value: "12".to_string(),
            namespace: "".to_string(),
        };
        let response3 = client.write(kv3).await.unwrap();
        assert!(response3.into_inner().success);

        let kv4 = KeyValue {
            key: "Vehicle.Infotainment.HVAC.OutdoorTemperature".to_string(),
            value: "20".to_string(),
            namespace: "".to_string(),
        };
        let response4 = client.write(kv4).await.unwrap();
        assert!(response4.into_inner().success);

        let kv5 = KeyValue {
            key: "Vehicle.Communication.Radio.Volume".to_string(),
            value: "10".to_string(),
            namespace: "".to_string(),
        };
        let response5 = client.write(kv5).await.unwrap();
        assert!(response5.into_inner().success);

        let kv6 = KeyValue {
            key: "test".to_string(),
            value: "test".to_string(),
            namespace: "".to_string(),
        };
        let response6 = client.write(kv6).await.unwrap();
        assert!(response6.into_inner().success);

        let kv7 = KeyValue {
            key: "Private.Info".to_string(),
            value: "test".to_string(),
            namespace: "AppName".to_string(),
        };
        let response7 = client.write(kv7).await.unwrap();
        assert!(response7.into_inner().success);
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes() {
        // list_nodes_starting_in('Vehicle.Infotainment', 1) -> ('Vehicle.Infotainment.Radio', 'Vehicle.Infotainment.HVAC')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle.Infotainment";
        let layers = 1;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(
            response.result,
            vec!["Vehicle.Infotainment.HVAC", "Vehicle.Infotainment.Radio"]
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_default_layers() {
        // list_nodes_starting_in('Vehicle.Infotainment') -> ('Vehicle.Infotainment.Radio', 'Vehicle.Infotainment.HVAC')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle.Infotainment";
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: None,
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(
            response.result,
            vec!["Vehicle.Infotainment.HVAC", "Vehicle.Infotainment.Radio"]
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_two_layers() {
        // list_nodes_starting_in('Vehicle.Infotainment', 2) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle.Infotainment";
        let layers = 2;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(
            response.result,
            vec![
                "Vehicle.Infotainment.HVAC.OutdoorTemperature",
                "Vehicle.Infotainment.Radio.CurrentStation",
                "Vehicle.Infotainment.Radio.Volume"
            ]
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_all_vehicle_keys() {
        // list_nodes_starting_in('Vehicle', 0) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume', 'Vehicle.Infotainment')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle";
        let layers = 0;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(
            response.result,
            vec![
                "Vehicle.Communication.Radio.Volume",
                "Vehicle.Infotainment",
                "Vehicle.Infotainment.HVAC.OutdoorTemperature",
                "Vehicle.Infotainment.Radio.CurrentStation",
                "Vehicle.Infotainment.Radio.Volume"
            ]
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_all_keys() {
        // list_nodes_starting_in('', 0) -> ('Vehicle.Infotainment.Radio.CurrentStation', 'Vehicle.Infotainment.Radio.Volume', 'Vehicle.Infotainment.HVAC.OutdoorTemperature', 'Vehicle.Communication.Radio.Volume', 'Vehicle.Infotainment', 'test')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "";
        let layers = 0;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(
            response.result,
            vec![
                "Vehicle.Communication.Radio.Volume",
                "Vehicle.Infotainment",
                "Vehicle.Infotainment.HVAC.OutdoorTemperature",
                "Vehicle.Infotainment.Radio.CurrentStation",
                "Vehicle.Infotainment.Radio.Volume",
                "test"
            ]
        );

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_empty_node() {
        // list_nodes_starting_in('', 1) -> ('Vehicle', 'test')

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "";
        let layers = 1;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(response.result, vec!["Vehicle", "test"]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_no_children() {
        // list_nodes_starting_in('Vehicle.Infotainment.Radio.Volume', 1) -> ()

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle.Infotainment.Radio.Volume";
        let layers = 1;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(response.result, vec![] as Vec<String>);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_negative_layers() {
        // list_nodes_starting_in('Vehicle', -1) -> ERROR

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle";
        let layers = -1;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(!response.success);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_node_does_not_exist() {
        // list_nodes_starting_in('Vehicle.DoesNotExist', 1) -> ERROR

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Vehicle.DoesNotExist";
        let layers = 1;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(!response.success);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_empty_string_always_exists() {
        // For empty data base: list_nodes_starting_in('', 1) -> ()

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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        // Act
        let node = "";
        let layers = 1;
        let namespace = "";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(response.result, vec![] as Vec<String>);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_nodes_nondefault_namespace() {
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

        // Initial clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();

        fill_db_example_tree(&mut client).await;

        // Act
        let node = "Private";
        let layers = 1;
        let namespace = "AppName";
        let response = client
            .nodes_starting_in(SubtreeInfo {
                node: node.to_string(),
                layers: Some(layers),
                namespace: namespace.to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // Assert
        assert!(response.success);
        assert_eq!(response.result, vec!["Private.Info"]);

        // Clean up.
        client.destroy_db(DestroyArguments {}).await.unwrap();
        server_task.abort();
    }
}
