use tokio::sync::{
    mpsc::channel,
    mpsc::{Receiver, Sender},
    oneshot,
};
use std::sync::OnceLock;

// ====> backup
use serde_json;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, AsyncSeekExt};
use std::collections::HashMap;
// ====> backup


struct SetKeyValueMessage {
    key: String,
    value: Vec<u8>,
    response: oneshot::Sender<()>,
}

struct GetKeyValueMessage {
    key: String,
    response: oneshot::Sender<Option<Vec<u8>>>,
}
struct DeleteKeyValueMessage {
    key: String,
    response: oneshot::Sender<()>,
}
enum KeyValueMessage {
    Get(GetKeyValueMessage),
    Delete(DeleteKeyValueMessage),
    Set(SetKeyValueMessage),
}

enum RoutingMessage {
    KeyValue(KeyValueMessage),
}


// ====> backup
enum WriterLogMessage {
    Set(String, Vec<u8>),
    Delete(String),
    Get(oneshot::Sender<HashMap<String, Vec<u8>>>),
}


impl WriterLogMessage {
    fn from_key_value_message(message: &KeyValueMessage) 
        -> Option<WriterLogMessage> {
        match message {
            KeyValueMessage::Get(_) => None,
            KeyValueMessage::Delete(message) => Some(
                WriterLogMessage::Delete(message.key.clone())
            ),
            KeyValueMessage::Set(message) => Some(
                WriterLogMessage::Set(message.key.clone(), message.value.clone())
            ),
        }
    }
}


async fn read_data_from_file(file_path: &str) -> io::Result<HashMap<String, Vec<u8>>> {
    let mut file = File::open(file_path).await?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).await?;
    let data: HashMap<String, Vec<u8>> = serde_json::from_str(&contents)?;
    Ok(data)
}

async fn load_map(file_path: &str) -> HashMap<String, Vec<u8>> {
    match read_data_from_file(file_path).await {
        Ok(data) => {
            println!("Data loaded from file: {:?}", data);
            return data
        },
        Err(e) => {
            println!("Failed to read from file: {:?}", e);
            println!("Starting with an empty hashmap.");
            return HashMap::new()
        }
    }
}


async fn key_value_actor(mut receiver: Receiver<KeyValueMessage>) {

    let (writer_key_value_sender, writer_key_value_receiver) = channel(32);
    tokio::spawn(writer_actor(writer_key_value_receiver));

    let (get_sender, get_receiver) = oneshot::channel();
    let _ = writer_key_value_sender.send(WriterLogMessage::Get(get_sender)).await;
    let mut map = get_receiver.await.unwrap();

    while let Some(message) = receiver.recv().await {
        if let Some(write_message) = WriterLogMessage::from_key_value_message(&message) {
            let _ = writer_key_value_sender.send(write_message).await;
        }
        match message {
            KeyValueMessage::Get(
                GetKeyValueMessage { key, response }
            ) => {
                let _ = response.send(map.get(&key).cloned());
            }
            KeyValueMessage::Delete(
                DeleteKeyValueMessage { key, response }
            ) => {
                map.remove(&key);
                let _ = response.send(());
            }
            KeyValueMessage::Set(
                SetKeyValueMessage { key, value, response }
            ) => {
                map.insert(key, value);
                let _ = response.send(());
            }
        }
    }
}


async fn writer_actor(mut receiver: Receiver<WriterLogMessage>) -> io::Result<()> {
    let mut map = load_map("./data.json").await;
    let mut file = File::create("./data.json").await.unwrap();

    while let Some(message) = receiver.recv().await {
        match message {
            WriterLogMessage::Set(key, value) => {
                map.insert(key, value);
            }
            WriterLogMessage::Delete(key) => {
                map.remove(&key);
            },
            WriterLogMessage::Get(response) => {
                let _ = response.send(map.clone());
            }
        }
        let contents = serde_json::to_string(&map).unwrap();
        file.set_len(0).await?;
        file.seek(std::io::SeekFrom::Start(0)).await?;
        file.write_all(contents.as_bytes()).await?;
        file.flush().await?; 
    }
    Ok(())
}


static ROUTER_SENDER: OnceLock<Sender<RoutingMessage>> = OnceLock::new();


async fn router(mut receiver: Receiver<RoutingMessage>) {
    let (key_value_sender, key_value_receiver) = channel(32);
    tokio::spawn(key_value_actor(key_value_receiver));

    while let Some(message) = receiver.recv().await {
        match message {
            RoutingMessage::KeyValue(message) => {
                let _ = key_value_sender.send(message).await;
            }
        }
    }
}
// ====> backup

pub async fn set(key: String, value: Vec<u8>) -> Result<(), std::io::Error> {
    let (tx, rx) = oneshot::channel();
    ROUTER_SENDER.get().unwrap().send(
        RoutingMessage::KeyValue(KeyValueMessage::Set(
            SetKeyValueMessage {
        key,
        value,
        response: tx,
    }))).await.unwrap();
    rx.await.unwrap();
    Ok(())
}


pub async fn get(key: String) -> Result<Option<Vec<u8>>, std::io::Error> {
    let (tx, rx) = oneshot::channel();
    ROUTER_SENDER.get().unwrap().send(
        RoutingMessage::KeyValue(KeyValueMessage::Get(
            GetKeyValueMessage {
        key,
        response: tx,
    }))).await.unwrap();
    Ok(rx.await.unwrap())
}


pub async fn delete(key: String) -> Result<(), std::io::Error> {
    let (tx, rx) = oneshot::channel();
    ROUTER_SENDER.get().unwrap().send(
        RoutingMessage::KeyValue(KeyValueMessage::Delete(
            DeleteKeyValueMessage {
        key,
        response: tx,
    }))).await.unwrap();
    rx.await.unwrap();
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let (sender, receiver) = channel(32);
    ROUTER_SENDER.set(sender).unwrap();

    tokio::spawn(router(receiver));

    // let _ = set("hello".to_string(), b"world".to_vec()).await?;
    let value = get("hello".to_string()).await?;
    println!("value: {:?}", value);
    // println!("value: {:?}", String::from_utf8(value.unwrap()));
    // let _ = delete("hello".to_string()).await?;
    let value = get("hello".to_string()).await?;
    println!("value: {:?}", value);
    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}
