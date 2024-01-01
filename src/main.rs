// Add dependencies to your Cargo.toml
// tokio = { version = "1", features = ["full"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1.0"

use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

use serde::{Serialize, Deserialize};
use serde_json;
// use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
enum Request {
    Get(String),
    Put(String, String),
}

#[derive(Serialize, Deserialize, Debug)]
enum Response {
    Ok(String),
    Err(String),
}

struct KeyValueStore {
    data: HashMap<String, String>,
}

impl KeyValueStore {
    fn new() -> Self {
        KeyValueStore {
            data: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn put(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
}

fn handle_client(mut stream: TcpStream, mut kv_store: KeyValueStore) {
    let mut stream_clone = stream.try_clone().expect("Failed to clone stream");
    let mut buffer = Vec::new();
    stream_clone.read_to_end(&mut buffer).expect("Failed to read from stream");

    let request: Request = serde_json::from_slice(&buffer).expect("Failed to deserialize request");

    let response = match request {
        Request::Get(key) => {
            if let Some(value) = kv_store.get(&key) {
                Response::Ok(value.clone())
            } else {
                Response::Err(format!("Key not found: {}", key))
            }
        }
        Request::Put(key, value) => {
            kv_store.put(key.clone(), value.clone());
            Response::Ok(format!("Key-Value pair added: {} - {}", key, value))
        }
    };

    let response_bytes = serde_json::to_vec(&response).expect("Failed to serialize response");
    stream.write_all(&response_bytes).expect("Failed to write to stream");
}

fn main() {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).expect("Failed to bind to address");

    println!("Server listening on {}", addr);

    let kv_store = KeyValueStore::new();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let kv_store_clone = kv_store.clone();
                thread::spawn(move || handle_client(stream, kv_store_clone));
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
