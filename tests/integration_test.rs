mod support;

use std::time::Duration;

use dogstatsd::{BatchingOptions, Client, OptionsBuilder};
use tokio::{sync::mpsc::Receiver, time::timeout};

use crate::support::TestServer;

#[tokio::test(flavor = "multi_thread")]
async fn simple_metric_test() {
    let server = TestServer::new("127.0.0.1:8126".into()).await;
    let opts = OptionsBuilder::new()
        .to_addr("127.0.0.1:8126".into())
        .build();
    let client = Client::new(opts).unwrap();

    let mut promise: Receiver<()>;
    {
        let mut shared = server.lock().unwrap();
        promise = shared.next_message_received();
    }
    client
        .gauge("my_stat", "7", ["tag1:value1"])
        .expect("unable to send stat");

    if timeout(Duration::from_secs(1), promise.recv())
        .await
        .is_err()
    {
        panic!("Didn't receive next message within a second");
    }

    {
        assert_eq!(
            server.lock().unwrap().last_metric().unwrap(),
            "my_stat:7|g|#tag1:value1"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn batching_test() {
    let server = TestServer::new("127.0.0.1:8127".into()).await;
    let opts = OptionsBuilder::new()
        .to_addr("127.0.0.1:8127".into())
        .batching_options(BatchingOptions {
            max_time: Duration::from_secs(2),
            max_buffer_size: 1024,
            max_retry_attempts: 0,
            initial_retry_delay: 25,
        })
        .build();
    let client = Client::new(opts).unwrap();

    let mut promise: Receiver<()>;
    {
        let mut shared = server.lock().unwrap();
        promise = shared.next_message_received();
    }
    client
        .gauge("my_stat", "7", ["tag1:value1"])
        .expect("unable to send stat");
    client
        .count("my_count", 29, ["tag1:value1"])
        .expect("unable to send stat");

    if timeout(Duration::from_secs(5), promise.recv())
        .await
        .is_err()
    {
        panic!("Didn't receive next batch within 5 seconds");
    }

    {
        assert_eq!(
            server.lock().unwrap().last_metric().unwrap(),
            "my_stat:7|g|#tag1:value1\nmy_count:29|c|#tag1:value1\n"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn batching_flushes_before_the_next_metric_exceeds_the_limit() {
    let server = TestServer::new("127.0.0.1:8128".into()).await;
    let max_buffer_size = 32;
    let opts = OptionsBuilder::new()
        .to_addr("127.0.0.1:8128".into())
        .batching_options(BatchingOptions {
            max_time: Duration::from_secs(30),
            max_buffer_size,
            max_retry_attempts: 0,
            initial_retry_delay: 25,
        })
        .build();
    let client = Client::new(opts).unwrap();

    let mut promise = server.lock().unwrap().next_message_received();
    client
        .gauge("first", "1", ["tag:value"])
        .expect("unable to send first stat");
    client
        .gauge("second", "2", ["tag:value"])
        .expect("unable to send second stat");
    drop(client);

    timeout(Duration::from_secs(1), promise.recv())
        .await
        .expect("didn't receive first packet within a second")
        .expect("notification channel closed before the first packet");
    timeout(Duration::from_secs(1), promise.recv())
        .await
        .expect("didn't receive second packet within a second")
        .expect("notification channel closed before the second packet");

    let shared = server.lock().unwrap();
    assert_eq!(
        shared.messages(),
        ["first:1|g|#tag:value\n", "second:2|g|#tag:value\n"]
    );
    assert!(shared
        .messages()
        .iter()
        .all(|packet| packet.len() <= max_buffer_size));
}
