use rocksdb::{Options, DB};

// static mut DB_INSTANCE: Option<DB> = None;

/*pub fn setup_db(path:&str) -> bool {
    println!("Set up database");
    unsafe { DB_INSTANCE = Some(DB::open_default(path).unwrap()) };
    true
}*/

#[derive(Debug)]
pub struct RocksDbFacade {
    db_instance: Option<DB>,
}

impl RocksDbFacade {
    pub fn new() -> RocksDbFacade {
        RocksDbFacade { db_instance: None }
    }

    pub fn open_db(&mut self, path: &str) -> Result<(), std::io::Error> {
        if self.db_instance.is_some() {
            return Ok(());
        } else {
            let mut opts = Options::default();
            opts.create_if_missing(true);
            self.db_instance = Some(
                DB::open(&opts, path)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?,
            );
            return Ok(());
        }
    }

    pub fn close_db(&mut self) -> Result<(), std::io::Error> {
        drop(self.db_instance.take());
        Ok(())
    }

    pub fn destroy_db(&mut self, path: &str) -> Result<(), std::io::Error> {
        DB::destroy(&Options::default(), path)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
    }

    pub fn write_db(&mut self, key: &str, value: &str) -> Result<(), std::io::Error> {
        let db_instance = self.db_instance.as_ref().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No database opened",
        ))?;
        db_instance
            .put(key, value)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
    }

    pub fn read_db(&mut self, key: &str) -> Result<String, std::io::Error> {
        let db_instance = self.db_instance.as_ref().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No database opened",
        ))?;
        let res = db_instance
            .get(key)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
        let value = res.ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Key not found",
        ))?;
        Ok(String::from_utf8_lossy(&value).to_string())
    }

    pub fn delete_db(&mut self, key: &str) -> Result<(), std::io::Error> {
        let db_instance = self.db_instance.as_ref().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No database opened",
        ))?;
        db_instance
            .delete(key)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
    }
}
