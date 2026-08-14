use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Starts listening on address and port
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    println!("Server listening on 127.0.0.1:7878");

    loop {
        let (_socket, addr) = listener.accept().await.unwrap();
        println!("New connection from {}", addr);

        tokio::spawn(async move {

        });
    }

    
    let store = Store::new();
    
    let key = "hello".to_string();
    let value = "world".to_string();

    store.set(key, value); // key/value ownership moves into set()

    let result = store.get("hello".to_string()); // fresh String , since original 'key' was moved above
    println!("{:?}", result); // {:?} becasue Option<String> doesn't implement Display 
}

// Thread-safe key-value store
// Wraps a HashMap so it can be shared and mutated across multiple threads
struct Store {
    // Arc: allows multiple threads to share ownership of the same data
    // Mutex: ensures only on thread can access/modify the HashMap at a time
    // HashMap: the actual key-value data
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Store {
    // Creates a new empty Store
    fn new() -> Store {
        Store {
            data: Arc::new(Mutex::new(HashMap::new())),
        }   
    }

    // Inserts or updates a key-value pair.
    // Takes ownership of both key and value.
    fn set(&self, key: String, value: String) {
        let mut map = self.data.lock().unwrap();
        map.insert(key, value);
    } // lock is automatically released when 'map' goes out of scope

    // Looks up value by key
    // Returns Some(value) if found, None if the key doesn't exist.
    fn get(&self, key: String) -> Option<String> {
        let map = self.data.lock().unwrap(); // lock for read-only access
        map.get(&key).cloned() // .cloned() copies the value out, since map.get() only gives a reference
    }

    // Removes the key-value pair if it exists
    // Returns the removed value (Some), or None if the key wasn't present.
    fn _delete(&self, key: String) -> Option<String> {
        let mut map = self.data.lock().unwrap(); // lock for exclusive (mutable) access, since remove() mutates the map
        map.remove(&key)
    }
}
