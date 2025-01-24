use tokio::{
    io::AsyncReadExt,
    sync::{
        mpsc::{channel, Receiver, Sender},
        oneshot,
    },
};

use serde_json::{self, value};
use tokio::fs::File;

use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::io::{self, AsyncBufReadExt, AsyncSeekExt, AsyncWriteExt};

static ROUTER_SENDER: OnceLock<Sender<RoutingMessage>> = OnceLock::new();

enum WriterLogMessage {
    Set(String, Vec<u8>),
    Delete(String),
    Get(oneshot::Sender<HashMap<String, Vec<u8>>>),
}

impl WriterLogMessage {
    fn from_key_value_message(message: &KeyValueMessage) -> Option<WriterLogMessage> {
        match message {
            KeyValueMessage::Get(_) => None,
            KeyValueMessage::Delete(message) => Some(WriterLogMessage::Delete(message.key.clone())),
            KeyValueMessage::Set(message) => Some(WriterLogMessage::Set(
                message.key.clone(),
                message.value.clone(),
            )),
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
        Ok(map) => {
            println!("Data loaded from file: {:?}", map);
            map
        }
        Err(e) => {
            println!("Faile to read from file: {:?}", e);
            println!("Starting with an empty hashmap.");
            HashMap::new()
        }
    }
}

async fn writer_actor(mut receiver: Receiver<WriterLogMessage>) -> io::Result<()> {
    let mut map = load_map("./data.json").await;
    let mut file = File::create("./data.json").await?;

    while let Some(message) = receiver.recv().await {
        match message {
            WriterLogMessage::Set(key, value) => {
                map.insert(key, value).unwrap();
            }
            WriterLogMessage::Delete(key) => {
                map.remove(&key);
            }
            WriterLogMessage::Get(response) => {
                let _ = response.send(map.clone());
            }
        };

        let contents = serde_json::to_string(&map).unwrap();
        file.set_len(0).await?;
        file.seek(std::io::SeekFrom::Start(0)).await?;
        file.write_all(contents.as_bytes()).await?;
        file.flush().await?;
    }

    Ok(())
}

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

async fn key_value_actor(mut receiver: Receiver<KeyValueMessage>) {
    let mut map: HashMap<String, Vec<u8>> = std::collections::HashMap::new();

    while let Some(message) = receiver.recv().await {
        match message {
            KeyValueMessage::Get(GetKeyValueMessage { key, response }) => {
                let _ = response.send(map.get(&key).cloned());
            }
            KeyValueMessage::Delete(DeleteKeyValueMessage { key, response }) => {
                map.remove(&key);
                let _ = response.send(());
            }
            KeyValueMessage::Set(SetKeyValueMessage {
                key,
                value,
                response,
            }) => {
                map.insert(key, value);
                let _ = response.send(());
            }
        }
    }
}

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

async fn set(key: String, value: Vec<u8>) -> Result<(), std::io::Error> {
    let (tx, rx) = oneshot::channel();
    ROUTER_SENDER
        .get()
        .unwrap()
        .send(RoutingMessage::KeyValue(KeyValueMessage::Set(
            SetKeyValueMessage {
                key,
                value,
                response: tx,
            },
        )))
        .await
        .unwrap();
    rx.await.unwrap();
    Ok(())
}

async fn get(key: String) -> Result<Option<Vec<u8>>, std::io::Error> {
    let (tx, rx) = oneshot::channel();

    ROUTER_SENDER
        .get()
        .unwrap()
        .send(RoutingMessage::KeyValue(KeyValueMessage::Get(
            GetKeyValueMessage { key, response: tx },
        )))
        .await
        .unwrap();
    Ok(rx.await.unwrap())
}

async fn delete(key: String) -> Result<(), std::io::Error> {
    let (tx, rx) = oneshot::channel();

    ROUTER_SENDER
        .get()
        .unwrap()
        .send(RoutingMessage::KeyValue(KeyValueMessage::Delete(
            DeleteKeyValueMessage { key, response: tx },
        )))
        .await
        .unwrap();
    rx.await.unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let (sender, receiver) = channel(32);
    ROUTER_SENDER.set(sender).unwrap();

    tokio::spawn(router(receiver));

    let _ = set("hello".to_string(), b"world".to_vec()).await?;

    let value = get("hello".to_string()).await?;

    println!("value: {:?}", String::from_utf8(value.unwrap()));

    let _ = delete("hello".to_string()).await?;

    let value = get("hello".to_string()).await?;

    println!("value: {:?}", value);

    Ok(())
}
