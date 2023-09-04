use std::sync::{Arc, Mutex};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};

pub struct TestServer {
    messages: Vec<String>,
    on_next_message: Option<Sender<()>>,
}

pub async fn create_server() -> Arc<Mutex<TestServer>> {
    let address = "127.0.0.1:8126".to_owned();

    let server = TestServer {
        messages: vec![],
        on_next_message: None,
    };
    let shared = Arc::new(Mutex::new(server));
    let shared_r = shared.clone();

    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let socket = UdpSocket::bind(address.clone())
            .await
            .expect(&format!("unable to bind to {:?}", address));

        // Signify that we're listening
        tx.send(()).unwrap();

        loop {
            // tokio::select!
            let mut buf = [0; 100];
            let (amt, _) = socket
                .recv_from(&mut buf)
                .await
                .expect("unable to read from socket");

            let on_next_message: Option<Sender<()>>;
            {
                let mut s = shared_r.lock().expect("unable to get server mutex");
                s.message_received(
                    String::from_utf8(buf[0..amt].to_vec())
                        .expect("unable to decode buffer to utf8 string"),
                );
                on_next_message = s.on_next_message.clone();
            }
            if let Some(p) = on_next_message {
                p.send(()).await.expect("unable to resolve promise");
            }
        }
    });

    // Wait for server to be listening
    let _ = rx.await;

    shared
}

impl TestServer {
    pub fn message_received(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn last_metric(&self) -> Option<&String> {
        self.messages.last()
    }

    pub fn next_message_received(&mut self) -> Receiver<()> {
        let (tx, rx) = mpsc::channel::<()>(1);
        self.on_next_message = Some(tx);
        rx
    }
}
