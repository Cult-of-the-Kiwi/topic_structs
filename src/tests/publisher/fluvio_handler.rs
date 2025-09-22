use fluvio::Fluvio;

use crate::{
    events::{
        DevcordEvent,
        message::{MessageEvent, MessageSent},
    },
    publisher::topic::fluvio::FluvioHandler,
    tests::publisher::test_notify,
};

async fn get_fluvio() -> Fluvio {
    Fluvio::connect().await.unwrap()
}

#[tokio::test]
pub async fn fluvio_test_notify() {
    let fluvio = get_fluvio().await;

    let handler: FluvioHandler<DevcordEvent> = FluvioHandler::new(fluvio);

    test_notify(handler).await;
}
