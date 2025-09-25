use std::time::Duration;

use tokio::{
    sync::mpsc::{self, Receiver},
    time::timeout,
};

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

async fn subscribe_receiver<T: EventPublisher<Event = DevcordEvent>>(
    publisher: &T,
    event: &DevcordEvent,
) -> anyhow::Result<Receiver<DevcordEvent>> {
    let (tx, rx) = mpsc::channel::<DevcordEvent>(1);

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
        .await
        .unwrap();

    Ok(rx)
}

pub async fn test_notify<T: EventPublisher<Event = DevcordEvent>>(
    publisher: T,
) -> anyhow::Result<()> {
    let event = DevcordEvent::MessageEvent(MessageEvent::MessageSentEvent(MessageSent {
        channel_id: String::from("Test"),
        sender: String::from("Test"),
        message: String::from("Test"),
    }));

    let mut rx = subscribe_receiver(&publisher, &event).await?;

    publisher.notify(event.clone()).await?;

    let Some(received_event) = rx.recv().await else {
        assert!(false);
        return Ok(());
    };
    assert_eq!(event, received_event);

    Ok(())
}

pub async fn test_subscribe_only_chosen_events<T: EventPublisher<Event = DevcordEvent>>(
    publisher: T,
) -> anyhow::Result<()> {
    let event_1 = DevcordEvent::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: String::from("Test"),
    }));

    let event_2 = DevcordEvent::AuthEvent(AuthEvent::UserCreatedEvent(UserCreated {
        id: String::from("Test"),
        username: String::from("Test"),
    }));

    let mut rx = subscribe_receiver(&publisher, &event_1).await?;

    publisher.notify(event_1.clone()).await?;

    let received_event = rx.recv().await.unwrap();
    assert_eq!(event_1, received_event);
    publisher.notify(event_2.clone()).await?;
    let received_event = timeout(Duration::from_secs(1), rx.recv()).await.ok();
    assert_eq!(None, received_event);

    Ok(())
}

pub async fn test_unsubscribe<T: EventPublisher<Event = DevcordEvent>>(
    publisher: T,
) -> anyhow::Result<()> {
    let event_1 = DevcordEvent::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: String::from("Test"),
    }));

    let event_2 = DevcordEvent::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: String::from("Test"),
    }));

    let mut rx = subscribe_receiver(&publisher, &event_1).await?;

    publisher.notify(event_1.clone()).await?;

    let received_event = rx.recv().await.unwrap();
    assert_eq!(event_1, received_event);
    publisher.unsubscribe(event_1).await?;
    publisher.notify(event_2.clone()).await?;
    let received_event = timeout(Duration::from_secs(1), rx.recv()).await.ok();
    assert_eq!(Some(None), received_event);

    Ok(())
}

pub async fn test_override_subscribe<T: EventPublisher<Event = DevcordEvent>>(
    publisher: T,
) -> anyhow::Result<()> {
    let event_1 = DevcordEvent::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: String::from("Test"),
    }));

    let event_2 = DevcordEvent::UserEvent(UserEvent::UserUpdatedEvent(UserUpdated {
        id: String::from("Test"),
    }));

    let mut rx_1 = subscribe_receiver(&publisher, &event_1).await?;

    publisher.notify(event_1.clone()).await?;

    let received_event = rx_1.recv().await.unwrap();
    assert_eq!(event_1, received_event);

    let (tx_2, mut rx_2) = mpsc::channel::<DevcordEvent>(1);

    publisher
        .subscribe(
            event_1.clone(),
            Box::new(move |event| {
                let tx = tx_2.clone();
                Box::pin(async move {
                    tx.send(event).await.unwrap();
                })
            }),
        )
        .await
        .unwrap();

    publisher.notify(event_2.clone()).await?;
    let received_event = timeout(Duration::from_secs(1), rx_1.recv()).await.ok();
    assert_eq!(Some(None), received_event);
    let received_event = rx_2.recv().await.unwrap();
    assert_eq!(event_2, received_event);

    Ok(())
}
