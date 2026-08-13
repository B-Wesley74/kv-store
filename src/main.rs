use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() {
    // (HashMap::new) make an empty map, (Mutex::new) wrap it for safe access, (Arc::new) wraps that so it can be shared
    let store: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    {
        let mut map = store.lock().unwrap(); // unwrap() because .lock() can fail 
        map.insert("hello".to_string(), "world".to_string());
    } // lock is released automatically

    {
        let map = store.lock().unwrap();
        println!("{:?}", map);
    }
}
