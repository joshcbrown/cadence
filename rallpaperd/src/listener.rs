use std::{
    io::Read,
    os::unix::net::{UnixListener, UnixStream},
};

use color_eyre::eyre::{Context, Result};
use tokio::sync::mpsc::Sender;

use crate::worker::WorkerMessage;

pub struct Listener {
    tx_worker: Sender<WorkerMessage>,
    listener: UnixListener,
}

async fn handle_connection(mut stream: UnixStream, tx: Sender<WorkerMessage>) {
    let mut message = String::new();
    stream.read_to_string(&mut message).unwrap();
    let worker_message = match message.as_str() {
        "change" => Some(WorkerMessage::ChangeWallpaper),
        "toggle" => Some(WorkerMessage::TogglePomo),
        "time" => Some(WorkerMessage::Time),
        _ => None,
    };
    if let Some(msg) = worker_message {
        tx.send(msg).await.unwrap()
    } else {
        println!("invalid message: {message}")
    }
}

impl Listener {
    pub fn new(tx_worker: Sender<WorkerMessage>) -> Result<Self> {
        let listener = UnixListener::bind("/tmp/rallpaper.sock")
            .context("couldn't establish message handler connection")?;
        Ok(Self {
            tx_worker,
            listener,
        })
    }

    pub async fn run(self) {
        for result in self.listener.incoming() {
            let stream = result.unwrap();
            tokio::spawn(handle_connection(stream, self.tx_worker.clone()));
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        std::fs::remove_file("/tmp/rallpaper.sock")
            .unwrap_or_else(|_| println!("problem destructing socket file"));
    }
}
