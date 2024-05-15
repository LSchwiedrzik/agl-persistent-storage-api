use rocksdb::{DB, Options};

static mut DB_INSTANCE: Option<DB> = None;

/*pub fn setup_db(path:&str) -> bool {
    println!("Set up database");
    unsafe { DB_INSTANCE = Some(DB::open_default(path).unwrap()) };
    true
}*/

pub fn open_db(path:&str) -> Result<(), std::io::Error> {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    unsafe {
        DB_INSTANCE = Some(DB::open(&opts, path)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?);
    }
    Ok(())
}

pub fn close_db() -> Result<(), std::io::Error> {
    unsafe { drop( DB_INSTANCE.take() )};
    Ok(())
}

pub fn destroy_db(path:&str) -> Result<(), std::io::Error> {
    DB::destroy(&Options::default(), path)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
}

pub fn write_db(key:&str, value:&str) -> Result<(), std::io::Error> {
    unsafe {
        let db_instance = DB_INSTANCE.as_ref().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "No database opened"))?;
        db_instance.put(key, value)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
    }
}

pub fn read_db(key:&str) -> Result<String, std::io::Error> {
    unsafe {
        let db_instance = DB_INSTANCE.as_ref().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "No database opened"))?;
        let res = db_instance.get(key)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
        let value = res.ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Key not found"))?;
        Ok(String::from_utf8_lossy(&value).to_string())
    }
}

pub fn delete_db(key:&str) -> Result<(), std::io::Error> {
    unsafe {
        let db_instance = DB_INSTANCE.as_ref().ok_or(std::io::Error::new(std::io::ErrorKind::Other, "No database opened"))?;
        db_instance.delete(key)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
    }
}