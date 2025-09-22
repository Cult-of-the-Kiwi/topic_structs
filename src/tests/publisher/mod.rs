use std::time::Duration;

use tokio::{sync::mpsc, time::timeout};

use crate::{
    events::{
        DevcordEvent,
        auth::{AuthEvent, UserCreated},
        message::{MessageEvent, MessageSent},
        user::{UserEvent, UserUpdated},
    },
    publisher::EventPublisher,
};

mod fluvio_handler;

pub async fn test_subscribe_only_chosen_events<T: EventPublisher<Event = DevcordEvent>>(
    publisher: T,
) {
    let event1 = DevcordEvent::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: String::from("Test"),
    }));

    let event2 = DevcordEvent::AuthEvent(AuthEvent::UserCreatedEvent(UserCreated {
        id: String::from("Test"),
        username: String::from("Test"),
    }));

    let (tx, mut rx) = mpsc::channel::<DevcordEvent>(1);

    publisher
        .subscribe(
            event1.clone(),
            Box::new(move |event| {
                let tx = tx.clone();
                Box::pin(async move {
                    tx.send(event).await.unwrap();
                })
            }),
        )
        .unwrap();

    publisher.notify(event1.clone()).unwrap();

    let received_event = rx.recv().await.unwrap();
    assert_eq!(event1, received_event);
    publisher.notify(event2.clone()).unwrap();
    let received_event = timeout(Duration::from_secs(1), rx.recv()).await.ok();
    assert_eq!(None, received_event)
}

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
