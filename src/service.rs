//use std::error::Error;

use crate::facade;

const DB_PATH: &str = "testpath";

/*pub fn setup_db(path:&str) -> bool {
    facade::setup_db(path)
}*/

pub fn open_db() -> (bool, String) {
    match facade::open_db(DB_PATH) {
        Ok(()) => return (true, String::from("Opened database at path '") + DB_PATH + "'"),
        Err(e) => return (false, String::from("Error when trying to open database at path '")
            + &DB_PATH + "': " + &e.to_string()),
    }
}

pub fn close_db() -> (bool, String) {
    match facade::close_db() {
        Ok(()) => return (true, String::from("Closed database")),
        Err(e) => return (false, String::from("Error when trying to close database: ") + &e.to_string()),
    }
}

pub fn destroy_db() -> (bool, String) {
    match facade::destroy_db(DB_PATH) {
        Ok(()) => return (true, String::from("Destroyed database at path '") + DB_PATH + "'"),
        Err(e) => return (false, String::from("Error when trying to destroy database at path '")
            + DB_PATH + "': " + &e.to_string()),
    }
}

pub fn write_db(key:&str, value:&str) -> (bool, String) {
    match facade::write_db(key, value) {
        Ok(()) => return (true, String::from("Wrote key '") + key + "' and value '" + value + "'"),
        Err(e) => return (false, String::from("Error when trying to write key '") + key
            + "' and value '" + value + "': " + &e.to_string()),
    }
}

pub fn read_db(key:&str) -> (bool, String, String) {
    match facade::read_db(key) {
        Ok(value) => return (true, String::from("Retrieved value '") + &value + "' from key '" + key + "'", value),
        Err(e) => return (false, String::from("Error when trying to retrieve from key '") + key
            + "': " + &e.to_string(), String::from("")),
    }
}

pub fn delete_db(key:&str) -> (bool, String) {
    match facade::delete_db(key) {
        Ok(()) => return (true, String::from("Deleted key '") + key + "'"),
        Err(e) => return (false, String::from("Error when trying to delete key '") + key
            + "': " + &e.to_string()),
    }
}