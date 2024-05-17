use crate::facade::RocksDbFacade;

const DB_PATH: &str = "testpath";

#[derive(Debug)]
pub struct DbService {
    rocks_db_facade: RocksDbFacade,
}

impl DbService {
    pub fn new() -> DbService {
        DbService { rocks_db_facade: RocksDbFacade::new() }
    }

    fn open_db(&mut self) -> (bool, String) {
        match self.rocks_db_facade.open_db(DB_PATH) {
            Ok(()) => return (true, String::from("Opened database at path '") + DB_PATH + "'"),
            Err(e) => return (false, String::from("Error when trying to open database at path '")
                + &DB_PATH + "': " + &e.to_string()),
        }
    }

    /*
    fn close_db() -> (bool, String) {
        match facade::close_db() {
            Ok(()) => return (true, String::from("Closed database")),
            Err(e) => return (false, String::from("Error when trying to close database: ") + &e.to_string()),
        }
    }
    */

    pub fn destroy_db(&mut self) -> (bool, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg);
        } 
        match self.rocks_db_facade.destroy_db(DB_PATH) {
            Ok(()) => return (true, String::from("Destroyed database at path '") + DB_PATH + "'"),
            Err(e) => return (false, String::from("Error when trying to destroy database at path '")
                + DB_PATH + "': " + &e.to_string()),
        }
    }

    pub fn write_db(&mut self, key:&str, value:&str) -> (bool, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg);
        } 
        match self.rocks_db_facade.write_db(key, value) {
            Ok(()) => return (true, String::from("Wrote key '") + key + "' and value '" + value + "'"),
            Err(e) => return (false, String::from("Error when trying to write key '") + key
                + "' and value '" + value + "': " + &e.to_string()),
        }
    }

    pub fn read_db(&mut self, key:&str) -> (bool, String, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg,  String::from(""));
        } 
        match self.rocks_db_facade.read_db(key) {
            Ok(value) => return (true, String::from("Retrieved value '") + &value + "' from key '" + key + "'", value),
            Err(e) => return (false, String::from("Error when trying to retrieve from key '") + key
                + "': " + &e.to_string(), String::from("")),
        }
    }

    pub fn check_if_key_exists(&mut self, key:&str) -> bool {
        match self.rocks_db_facade.read_db(key) {
            Ok(_value) => return true,
            Err(_e) => return false,
        }
    }

    pub fn delete_db(&mut self, key:&str) -> (bool, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg);
        } 

        if self.check_if_key_exists(key) {
            match self.rocks_db_facade.delete_db(key) {
                Ok(()) => return (true, String::from("Deleted key '") + key + "'"),
                Err(e) => return (false, String::from("Error when trying to delete key '") + key
                    + "': " + &e.to_string()),
            }
        } else {
            return (false, String::from("Key '") + key + "' does not exist!")
        }
    }
}