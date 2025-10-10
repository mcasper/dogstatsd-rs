mod support;

use std::time::Duration;

use dogstatsd::{BatchingOptions, Client, OptionsBuilder};
use tokio::{sync::mpsc::Receiver, time::{sleep, timeout}};

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
        .gauge("my_stat", "7", &["tag1:value1"])
        .expect("unable to send stat");
    client
        .count("my_count", 29, &["tag1:value1"])
        .expect("unable to send stat");

    // The batch processor requires a metric to be sent _after_ the timeout has been reached
    // to flush the buffer. Ideally there would be a separate timer running to automatically flush it,
    // but for now we'll make do with a sleep.
    sleep(Duration::from_secs(2)).await;

    client
        .timing("my_timing", 311, &["tag1:value1"])
        .expect("unable to send stat");

    if let Err(_) = timeout(Duration::from_secs(5), promise.recv()).await {
        assert!(false, "Didn't receive next batch within 5 seconds");
    }

    {
        assert_eq!(
            server.lock().unwrap().last_metric().unwrap(),
            "my_stat:7|g|#tag1:value1\nmy_count:29|c|#tag1:value1\nmy_timing:311|ms|#tag1:value1\n"
        );
    }
}
