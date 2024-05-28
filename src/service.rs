use crate::facade::RocksDbFacade;

const DB_PATH: &str = "rocksdb";

#[derive(Debug)]
pub struct DbService {
    rocks_db_facade: RocksDbFacade,
}

impl DbService {
    pub fn new() -> DbService {
        DbService {
            rocks_db_facade: RocksDbFacade::new(),
        }
    }

    fn open_db(&mut self) -> (bool, String) {
        match self.rocks_db_facade.open_db(DB_PATH) {
            Ok(()) => {
                return (
                    true,
                    String::from("Opened database at path '") + DB_PATH + "'",
                )
            }
            Err(e) => {
                return (
                    false,
                    String::from("Error when trying to open database at path '")
                        + &DB_PATH
                        + "': "
                        + &e.to_string(),
                )
            }
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
            Ok(()) => {
                return (
                    true,
                    String::from("Destroyed database at path '") + DB_PATH + "'",
                )
            }
            Err(e) => {
                return (
                    false,
                    String::from("Error when trying to destroy database at path '")
                        + DB_PATH
                        + "': "
                        + &e.to_string(),
                )
            }
        }
    }

    pub fn write_db(&mut self, key: &str, value: &str, namespace: &str) -> (bool, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg);
        }
        if key.is_empty() {
            return (
                false,
                String::from("Error when trying to write key '")
                    + key
                    + "' and value '"
                    + value
                    + "': Key cannot be empty string.",
            );
        }

        let namespace_key = format!("{namespace}_.{key}");
        match self.rocks_db_facade.write_db(namespace_key.as_str(), value) {
            Ok(()) => {
                return (
                    true,
                    String::from("Wrote key '") + key + "' and value '" + value + "' in namespace '" + namespace + "'",
                )
            }
            Err(e) => {
                return (
                    false,
                    String::from("Error when trying to write key '")
                        + key
                        + "' and value '"
                        + value
                        + "' in namespace '"
                        + namespace
                        + "': "
                        + &e.to_string(),
                )
            }
        }
    }

    pub fn read_db(&mut self, key: &str, namespace: &str) -> (bool, String, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg, String::from(""));
        }
        let namespace_key = format!("{namespace}_.{key}");
        match self.rocks_db_facade.read_db(namespace_key.as_str()) {
            Ok(value) => {
                return (
                    true,
                    String::from("Retrieved value '") + &value + "' from key '" + key + "' in namespace '" + namespace + "'",
                    value,
                )
            }
            Err(e) => {
                return (
                    false,
                    String::from("Error when trying to retrieve from key '")
                        + key
                        + "' in namespace '"
                        + namespace
                        + "': "
                        + &e.to_string(),
                    String::from(""),
                )
            }
        }
    }

    pub fn check_if_key_exists(&mut self, key: &str, namespace: &str) -> bool {
        let namespace_key = format!("{namespace}_.{key}");
        match self.rocks_db_facade.read_db(namespace_key.as_str()) {
            Ok(_value) => return true,
            Err(_e) => return false,
        }
    }

    pub fn delete_db(&mut self, key: &str, namespace: &str) -> (bool, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg);
        }

        if self.check_if_key_exists(key, namespace) {
            let namespace_key = format!("{namespace}_.{key}");
            match self.rocks_db_facade.delete_db(&namespace_key.as_str()) {
                Ok(()) => return (true, String::from("Deleted key '") + key + "' in namespace '" + namespace + "'"),
                Err(e) => {
                    return (
                        false,
                        String::from("Error when trying to delete key '")
                            + key
                            + "' in namespace '"
                            + namespace
                            + "': "
                            + &e.to_string(),
                    )
                }
            }
        } else {
            return (false, String::from("Key '") + key + "' does not exist in namespace '" + namespace + "'!");
        }
    }

    pub fn search_db(&mut self, substring: &str, namespace: &str) -> (bool, String, Vec<String>) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg, Vec::new());
        }
        let namespace_prefix = format!("{namespace}_.");
        match self.rocks_db_facade.list_keys_with_prefix(namespace_prefix.as_str()) {
            Ok(value) => {

                //let namespace_prefix = format!("{namespace}_.");
                //let mut res: Vec<String> = value.iter_mut()
                //for string in value.iter_mut() {
                //    string = string.strip_prefix(namespace_prefix.as_str());
                //}
                let mut res = value
                    .into_iter()
                    .filter(|string| string
                        .contains(substring)/*&& string.starts_with(namespace_prefix.as_str())*/)
                    /*.map(|string| string //THIS IS THE MOST RECENT ATTEMPT - BUYER BEWARE!
                        .strip_prefix(namespace_prefix.as_str()).expect("Weird error!"))*/
                    .collect::<Vec<String>>();
                /*for x in &mut res {
                    for y in x {
                        if x.contains(substring) {
                            x = x.strip_prefix(namespace_prefix.to_str())
                        }
                    }
                }*/
                
                /*for string in &res {
                    //string = string.replace(string, "");
                    string = string.strip_prefix(namespace_prefix.as_str()).to_string();
                }*/

                res.sort();
                //res = res.into_iter().map(|string| string.strip_prefix("_.")).collect();
                return (
                    true,
                    String::from("Retrieved list of keys containing substring '") + substring + "' in namespace '" + namespace + "'",
                    res,
                );
            } //s.retain(|c| !r#"(),".;:'"#.contains(c));
            Err(e) => {
                return (
                    false,
                    String::from("Error when trying to search for keys containing '")
                        + substring
                        + "' in namespace '"
                        + namespace
                        + "': "
                        + &e.to_string(),
                    Vec::new(),
                )
            }
        }
    }

    pub fn delete_recursively_from_db(&mut self, node: &str, namespace: &str) -> (bool, String) {
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg);
        }

        if node.is_empty() {
            return (false, "Error: Key String was empty!".to_string());
        }

        let mut deleted_keys = "Deleted Keys: ".to_string();

        let namespace_node = format!("{namespace}_.{node}");
        match self.rocks_db_facade.list_keys_with_prefix(namespace_node.as_str()){
            Ok(res) => {
                for key in res {
                    let namespace_key = format!("{namespace}_.{key}");
                    match self.rocks_db_facade.delete_db(&namespace_key.as_str()) {
                        Ok(()) => deleted_keys = format!("{} {}", deleted_keys, key),
                        Err(_e) => {
                            return (false, "Error deleting key '".to_string() + &key + "'.")
                        }
                    }
                }
                return (
                    true,
                    "Successfully deleted nodes: ".to_string() + &deleted_keys + ".",
                );
            }
            Err(_e) => (
                false,
                "Error when trying to list keys with prefix '".to_string() + node + "'",
            ),
        }
    }

    pub fn nodes_starting_in(
        &mut self,
        node: &str,
        layers: Option<i32>,
    ) -> (bool, String, Vec<String>) {
        let l = layers.unwrap_or(1);
        if l < 0 {
            return (
                false,
                String::from("Error when trying to list nodes starting in '")
                    + node
                    + "' exactly "
                    + &l.to_string()
                    + " layers deep: layers must be non-negative",
                Vec::new(),
            );
        }
        let (is_open, msg) = self.open_db();
        if !is_open {
            return (false, msg, Vec::new());
        }
        let mut node_dot = String::from(node);
        if !node.is_empty() {
            node_dot.push('.');
        }
        match self.rocks_db_facade.list_keys_with_prefix(&node_dot) {
            Ok(mut value) => {
                if l == 0 {
                    if self.check_if_key_exists(node) {
                        value.push(String::from(node));
                    }
                    if value.is_empty() && !node.is_empty() {
                        return (
                            false,
                            String::from("Error when trying to list nodes starting in '")
                                + node
                                + "' exactly "
                                + &l.to_string()
                                + " layers deep: node '"
                                + node
                                + "' doesn't exist",
                            Vec::new(),
                        );
                    }
                    value.sort();
                    return (
                        true,
                        String::from("Retrieved list of keys starting in '")
                            + node
                            + "' any number of layers deep (special case layers = '0')",
                        value,
                    );
                } else {
                    if value.is_empty() && !node.is_empty() && !self.check_if_key_exists(node) {
                        return (
                            false,
                            String::from("Error when trying to list nodes starting in '")
                                + node
                                + "' exactly "
                                + &l.to_string()
                                + " layers deep: node '"
                                + node
                                + "' doesn't exist",
                            Vec::new(),
                        );
                    }
                    let total_depth: i32 =
                        node_dot.chars().filter(|&c| c == '.').count() as i32 - 1 + l;
                    let mut res: Vec<String> = Vec::new();
                    for key in value.iter_mut() {
                        let mut count = 0;
                        for (i, c) in key.chars().enumerate() {
                            if c == '.' {
                                count += 1;
                                if count > total_depth {
                                    res.push(key[..i].to_string());
                                    break;
                                }
                            }
                        }
                        if count == total_depth {
                            res.push(key.to_string());
                        }
                    }
                    res.sort();
                    res.dedup();
                    return (
                        true,
                        String::from("Retrieved list of nodes starting in '")
                            + node
                            + "' exactly "
                            + &l.to_string()
                            + " layers deep",
                        res,
                    );
                }
            }
            Err(e) => {
                return (
                    false,
                    String::from("Error when trying to list nodes starting in '")
                        + node
                        + "' exactly "
                        + &l.to_string()
                        + " layers deep: "
                        + &e.to_string(),
                    Vec::new(),
                )
            }
        }
    }
}
