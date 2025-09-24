use crate::{
    events::DevcordEvent,
    publisher::topic::fluvio::FluvioHandler,
    tests::publisher::{test_notify, test_subscribe_only_chosen_events},
};

#[tokio::test]
pub async fn fluvio_test_notify() {
    //let handler = FluvioStack::get_fluvio_handler().await.unwrap();
    let handler: FluvioHandler<DevcordEvent> = FluvioHandler::new(None).await.unwrap();

    test_notify(handler).await.unwrap();
}

#[tokio::test]
pub async fn fluvio_test_subscribe_only_chosen_events() {
    let handler: FluvioHandler<DevcordEvent> = FluvioHandler::new(None).await.unwrap();

    test_subscribe_only_chosen_events(handler).await.unwrap();
}
