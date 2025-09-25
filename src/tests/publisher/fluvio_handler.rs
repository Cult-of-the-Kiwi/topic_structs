use std::time::Duration;

use serial_test::serial;
use tokio::time::sleep;

use crate::{
    events::DevcordEvent,
    publisher::topic::fluvio::FluvioHandler,
    tests::publisher::{
        test_notify, test_override_subscribe, test_subscribe_only_chosen_events, test_unsubscribe,
    },
};

const TEST_TIMEOUT: u64 = 400;

async fn test<T: AsyncFnOnce(FluvioHandler<DevcordEvent>) -> anyhow::Result<()>>(
    test: T,
) -> anyhow::Result<()> {
    sleep(Duration::from_millis(TEST_TIMEOUT)).await;
    let handler: FluvioHandler<DevcordEvent> = FluvioHandler::new(None).await.unwrap();
    handler.reset_fluvio().await.unwrap();

    test(handler).await
}

#[tokio::test]
#[serial]
pub async fn fluvio_test_notify() {
    test(test_notify).await.unwrap();
}

#[tokio::test]
#[serial]
pub async fn fluvio_test_subscribe_only_chosen_events() {
    test(test_subscribe_only_chosen_events).await.unwrap();
}

#[tokio::test]
#[serial]
pub async fn fluvio_test_unsubscribe() {
    test(test_unsubscribe).await.unwrap();
}

#[tokio::test]
#[serial]
pub async fn fluvio_test_override_subscribe() {
    test(test_override_subscribe).await.unwrap();
}
