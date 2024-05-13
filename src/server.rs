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
use crate::storage_api::{
    Key, Value, KeyValue, StandardResponse, ReadResponse, SetupArguments, DestroyArguments, 
};

use crate::service;

//const NO_KEY_ERR: &str = "the key doesn't exist in the database";

#[derive(Debug)]
pub struct DatabaseManager {

}

#[tonic::async_trait]
impl Database for DatabaseManager {
    //setup_db, destroy_db, write, read, delete
    async fn setup_db(
        &self,
        _request: Request<SetupArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: bool = service::setup_db("testpath".into());

        Ok(Response::new(StandardResponse {
            success: res,
            message: "success".into(),
        }))
    }

    async fn destroy_db(
        &self,
        _request: Request<DestroyArguments>,
    ) -> Result<Response<StandardResponse>, Status> {
        let res: bool = service::destroy_db("testpath".into());

        Ok(Response::new(StandardResponse {
            success: res,
            message: "success".into(),
        }))
    }

    async fn write(
        &self,
        request: Request<KeyValue>,
    ) -> Result<Response<StandardResponse>, Status> {
        let keyvalue = request.into_inner();
        let res: bool = service::write_db(&keyvalue.key, &keyvalue.value);

        Ok(Response::new(StandardResponse {
            success: res,
            message: "success".into(),
        }))
    }

    async fn read(
        &self,
        request: Request<Key>,
    ) -> Result<Response<ReadResponse>, Status> {
        let key = request.into_inner();
        let res = service::read_db(&key.key);

        Ok(Response::new(ReadResponse {
            success: res.0,
            message: "success".into(),
            result: Some(Value{value: res.1}),
        }))
    }

    async fn delete(
        &self,
        request: Request<Key>,
    ) -> Result<Response<StandardResponse>, Status> {
        let key = request.into_inner();
        let res: bool = service::delete_db(&key.key);

        Ok(Response::new(StandardResponse {
            success: res,
            message: "success".into(),
        }))
    }
}
