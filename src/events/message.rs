use fluvio::RecordKey;
use serde::{Deserialize, Serialize};

use crate::publisher::topic::{TopicEvent, fluvio::KeyEvent};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MessageSent {
    pub channel_id: String,
    pub sender: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum MessageEvent {
    #[serde(rename = "message_sent")]
    MessageSentEvent(MessageSent),
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
