use fluvio::RecordKey;
use serde::{Deserialize, Serialize};

use crate::{
    events::DevcordEventType,
    publisher::{
        TypedEvent,
        topic::{TopicEvent, fluvio::KeyEvent},
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageSent {
    pub channel_id: String,
    pub sender: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind")]
pub enum MessageEvent {
    #[serde(rename = "message_sent")]
    MessageSentEvent(MessageSent),
}

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub enum MessageEventType {
    Message,
}

impl TopicEvent for MessageEvent {
    fn event_topic(&self) -> crate::publisher::topic::Topic {
        match self {
            MessageEvent::MessageSentEvent(_) => String::from("message"),
        }
    }
}

impl KeyEvent for MessageEvent {
    fn event_key(&self) -> fluvio::RecordKey {
        match self {
            MessageEvent::MessageSentEvent(_) => RecordKey::NULL,
        }
    }
}

impl TypedEvent for MessageEvent {
    type EventType = DevcordEventType;

    fn event_type(&self) -> Self::EventType {
        DevcordEventType::Message(match self {
            MessageEvent::MessageSentEvent(_) => MessageEventType::Message,
        })
    }
}
