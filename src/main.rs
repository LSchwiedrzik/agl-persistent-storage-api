use rocksdb::{DB, Options};

fn main() {
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