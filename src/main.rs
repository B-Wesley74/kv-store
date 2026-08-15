use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// Thread-safe key-value store
// Wraps a HashMap so it can be shared and mutated across multiple threads
#[derive(Clone)]
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
    fn delete(&self, key: String) -> Option<String> {
        let mut map = self.data.lock().unwrap(); // lock for exclusive (mutable) access, since remove() mutates the map
        map.remove(&key)
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    println!("Server listening on 127.0.0.1:7878");

    // Create ONE store, shared by every client connection.
    // This lives for the whole lidetime of the server.
    let store = Store::new();

    loop {
        // socket = the actual connection to this one client
        // addr = who connected (their IP/port)
        let (socket, addr) = listener.accept().await.unwrap();
        println!("New connection from {}", addr);

        // each spawned task gets its own Store "handle", but they all
        // point at the same underlying Arc<Mutex<HashMap>>.
        let store_for_this_client = store.clone();

        // Spawn this connection's handling as and independent task,
        // so the loop can immediately go back to accept() the next client
        // instead of waiting for this one to finish.
        tokio::spawn(async move {
            handle_connection(socket, store_for_this_client).await;
        });
    }
}

async fn handle_connection(socket: tokio::net::TcpStream, store: Store) {
    // Split the conection into a reader half and writer half,
    // so we can read incoming lines and write responses independenly.
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Keep handling commands from this client until they disconnect.
    loop {

        line.clear(); // reuse the same String each loop, avoid reallocating 

        // read_line waits (asynchronously) for the client to send a line of text.
        let bytes_read = reader.read_line(&mut line).await.unwrap();
        if bytes_read == 0 {
            println!("Line disconnected");
            break;  // client disconnected
        }

        // Split "SET hello world" into ["SET", "hello", "world"]
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        // Guard against an empty line (e.g. client just hit enter)
        if parts.is_empty() {
            writer.write_all(b"ERROR empty command\n").await.unwrap();
            continue;   // skip the rest of this loop iteration, go read the next line
        }

        match parts[0] {
            "SET" => {
                // SET needs exactly 3 parts (SET, key, value)
                if parts.len() != 3 {
                    writer.write_all(b"ERROR usage SET key value\n").await.unwrap();
                } else {
                    // parts 2 & 3 are &str (borrowed) - Store::set wants owned String.
                    // so convert with .to_string()
                    store.set(parts[1].to_string(), parts[2].to_string());
                    writer.write_all(b"OK\n").await.unwrap();
                }
            }
            "GET" => {
                // GET needs exactly 2 parts: GET, key
                if parts.len() != 2 {
                    writer.write_all(b"ERROR usage: GET key\n").await.unwrap();
                } else {
                    match store.get(parts[1].to_string()) {
                        Some(value) => {
                            let response = format!("{}\n", value);
                            writer.write_all(response.as_bytes()).await.unwrap();
                        }
                        None => {
                            writer.write_all(b"NOT FOUND\n").await.unwrap();
                        }
                    }
                }
            }
            "DELETE" => {
                if parts.len() != 2 {
                    writer.write_all(b"ERROR usage: DELETE key\n").await.unwrap();
                } else {
                    match store.delete(parts[1].to_string()) {
                        Some(_) => writer.write_all(b"OK\n").await.unwrap(),
                        None => writer.write_all(b"NOT FOUND\n").await.unwrap(),
                    }
                }
            }
            // Fall back for that is not SET/GET/DELETE.
            // match must be exhaustive, so the '_' catches everything else.
            _ => {
                writer.write_all(b"ERROR unknown command\n").await.unwrap();
            }
        }
    }
}




