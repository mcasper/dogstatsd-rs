mod support;

use std::time::Duration;

use dogstatsd::{Client, OptionsBuilder};
use tokio::{sync::mpsc::Receiver, time::timeout};

use crate::support::create_server;

fn create_client() -> Client {
    let opts = OptionsBuilder::new()
        .to_addr("127.0.0.1:8126".into())
        .from_addr("127.0.0.1:8127".into())
        .build();
    Client::new(opts).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn whole_test() {
    let server = create_server().await;
    let client = create_client();

    let mut promise: Receiver<()>;
    {
        let mut shared = server.lock().unwrap();
        promise = shared.next_message_received();
    }
    client
        .gauge("my_stat", "7", &["tag1:value1"])
        .expect("unable to send stat");

    if let Err(_) = timeout(Duration::from_secs(1), promise.recv()).await {
        assert!(false, "Didn't receive next message within a second");
    }

    {
        assert_eq!(
            server.lock().unwrap().last_metric().unwrap(),
            "my_stat:7|g|#tag1:value1"
        );
    }
}
