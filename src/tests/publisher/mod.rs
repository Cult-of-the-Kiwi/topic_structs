use crate::{
    events::{
        DevcordEvent,
        message::{MessageEvent, MessageSent},
    },
    publisher::{EventPublisher, TypedEvent},
};

mod fluvio_handler;

pub async fn test_notify<T: EventPublisher<Event = DevcordEvent>>(publisher: T) {
    let event = DevcordEvent::MessageEvent(MessageEvent::MessageSentEvent(MessageSent {
        channel_id: String::from("Test"),
        sender: String::from("Test"),
        message: String::from("Test"),
    }));

    publisher
        .subscribe(event.clone(), Box::new(|event| Box::pin(async move {})))
        .unwrap();

    publisher.notify(event).unwrap();
}
