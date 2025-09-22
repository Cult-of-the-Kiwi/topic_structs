use tokio::sync::mpsc;

use crate::{
    events::{
        DevcordEvent,
        message::{MessageEvent, MessageSent},
    },
    publisher::EventPublisher,
};

mod fluvio_handler;

pub async fn test_notify<T: EventPublisher<Event = DevcordEvent>>(publisher: T) {
    let event = DevcordEvent::MessageEvent(MessageEvent::MessageSentEvent(MessageSent {
        channel_id: String::from("Test"),
        sender: String::from("Test"),
        message: String::from("Test"),
    }));

    let (tx, mut rx) = mpsc::channel::<DevcordEvent>(1);

    publisher
        .subscribe(
            event.clone(),
            Box::new(move |event| {
                let tx = tx.clone();
                Box::pin(async move {
                    tx.send(event).await.unwrap();
                })
            }),
        )
        .unwrap();

    publisher.notify(event.clone()).unwrap();

    let received_event = rx.recv().await.unwrap();
    assert_eq!(event, received_event)
}
