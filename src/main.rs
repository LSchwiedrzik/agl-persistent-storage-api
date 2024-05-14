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

/*fn main() {
    //grpc_main();
    //return;

    // NB: db is automatically closed at end of lifetime
    let path = "testpath";
    {
        let db = DB::open_default(path).unwrap();
        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();
        db.put(b"c", b"3").unwrap();
        match db.get(b"a") {
            Ok(Some(value)) => println!("retrieved value {}", String::from_utf8(value).unwrap()),
            Ok(None) => println!("value not found"),
            Err(e) => println!("operational problem encountered: {}", e),
        }
        db.delete(b"a").unwrap();
        match db.get(b"a") {
            Ok(Some(value)) => println!("retrieved value {}", String::from_utf8(value).unwrap()),
            Ok(None) => println!("value not found"),
            Err(e) => println!("operational problem encountered: {}", e),
        }
    }
    let _ = DB::destroy(&Options::default(), path);
    println!("Hello, world!");
}*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9001".parse()?;
    //let dbmanager = DatabaseManager::default();
    let dbmanager = DatabaseManager {};

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

fn put() {
    println!("Put data into database");
}

fn get() {
    println!("Get data from database");
}

#[cfg(test)]
// Unit tests go here
mod tests {

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
