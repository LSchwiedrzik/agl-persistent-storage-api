//use futures::Stream;
//use std::borrow::BorrowMut;
//use std::collections::HashMap;
//use std::pin::Pin;
//use std::sync::Arc;
//use tokio::sync::{mpsc, Mutex};
//use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{Request, Response, Status};
//use rocksdb::{Options, DB};

use crate::storage_api::database_server::Database;
//use crate::storage_api::database_client::DatabaseClient;
use crate::storage_api::{
    Key, KeyValue, StandardResponse, ReadResponse, OpenArguments, CloseArguments, DestroyArguments, 
};

use crate::service;

//const NO_KEY_ERR: &str = "the key doesn't exist in the database";

#[derive(Debug)]
pub struct DatabaseManager {

}

#[tonic::async_trait]
impl Database for DatabaseManager {
    //open_db, close_db, destroy_db, write, read, delete
    /*async fn setup_db(
        &self,
        _request: Request<SetupArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: bool = service::setup_db("testpath".into());

        Ok(Response::new(StandardResponse {
            success: res,
            message: "success".into(),
        }))
    }*/

    async fn open_db(
        &self,
        _request: Request<OpenArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: (bool, String) = service::open_db();

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn close_db(
        &self,
        _request: Request<CloseArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: (bool, String) = service::close_db();

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn destroy_db(
        &self,
        _request: Request<DestroyArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: (bool, String) = service::destroy_db();

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
        let res: (bool, String) = service::write_db(&keyvalue.key, &keyvalue.value);

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }

    async fn read(
        &self,
        request: Request<Key>,
    ) -> Result<Response<ReadResponse>, Status> {
        let key: Key = request.into_inner();
        let res: (bool, String, String) = service::read_db(&key.key);

        Ok(Response::new(ReadResponse {
            success: res.0,
            message: res.1,
            result: res.2,
        }))
    }

    async fn delete(
        &self,
        request: Request<Key>,
    ) -> Result<Response<StandardResponse>, Status> {
        let key = request.into_inner();
        let res: (bool, String) = service::delete_db(&key.key);

        Ok(Response::new(StandardResponse {
            success: res.0,
            message: res.1,
        }))
    }
}



/*#[cfg(test)]
// Unit tests go here
mod tests {
    use crate::storage_api::OpenArguments;
    

    #[test]
    fn it_works() {
        let mut client = DatabaseClient::connect("http://127.0.0.1:9001").await?;

        let oa = OpenArguments { };

        let request1 = tonic::Request::new(oa);
        let response1 = client.open_db(request1).await?;

        let kv = KeyValue {
            key: String::from("testkey".into),
            value: String::from("testvalue".into),
        };

        let request2 = tonic::Request::new(kv);
        let response2 = client.write(request2).await?;

        let ky = Key {
            key: String::from("testkey".into),
        };

        let request3 = tonic::Request::new(ky);
        let response3 = client.read(request3).await?;

        assert_eq!(response.into_inner().result.value, "testvalue");
    }
}*/
