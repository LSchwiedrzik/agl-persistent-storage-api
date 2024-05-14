use crate::facade;

/*pub fn setup_db(path:&str) -> bool {
    facade::setup_db(path)
}*/

pub fn open_db(path:&str) -> bool {
    facade::open_db(path)
}

pub fn close_db() -> bool {
    facade::close_db()
}

pub fn destroy_db(path:&str) -> bool {
    facade::destroy_db(path)
}

pub fn write_db(key:&str, value:&str) -> bool {
    facade::write_db(key, value)
}

pub fn read_db(key:&str) -> (bool, String) {
    facade::read_db(key)
}

pub fn delete_db(key:&str) -> bool {
    facade::delete_db(key)
}