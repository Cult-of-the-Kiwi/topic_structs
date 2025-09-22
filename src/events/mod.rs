use serde::{Deserialize, Serialize};

use crate::{
    events::{
        auth::{AuthEvent, AuthEventType},
        group::{GroupEvent, GroupEventType},
        message::{MessageEvent, MessageEventType},
        user::{UserEvent, UserEventType},
    },
    publisher::{
        TypedEvent,
        topic::{TopicEvent, fluvio::KeyEvent},
    },
};

pub mod auth;
pub mod group;
pub mod message;
pub mod user;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "service")]
pub enum DevcordEvent {
    #[serde(rename = "user")]
    UserEvent(UserEvent),
    #[serde(rename = "auth")]
    AuthEvent(AuthEvent),
    #[serde(rename = "message")]
    MessageEvent(MessageEvent),
    #[serde(rename = "group")]
    GroupEvent(GroupEvent),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DevcordEventType {
    User(UserEventType),
    Auth(AuthEventType),
    Message(MessageEventType),
    Group(GroupEventType),
}

impl TopicEvent for DevcordEvent {
    fn event_topic(&self) -> crate::publisher::topic::Topic {
        match self {
            DevcordEvent::UserEvent(event) => event.event_topic(),
            DevcordEvent::AuthEvent(event) => event.event_topic(),
            DevcordEvent::MessageEvent(event) => event.event_topic(),
            DevcordEvent::GroupEvent(event) => event.event_topic(),
        }
    }
}

impl KeyEvent for DevcordEvent {
    fn event_key(&self) -> fluvio::RecordKey {
        match self {
            DevcordEvent::UserEvent(event) => event.event_key(),
            DevcordEvent::AuthEvent(event) => event.event_key(),
            DevcordEvent::MessageEvent(event) => event.event_key(),
            DevcordEvent::GroupEvent(event) => event.event_key(),
        }
    }
}

impl TypedEvent for DevcordEvent {
    type EventType = DevcordEventType;

    fn event_type(&self) -> Self::EventType {
        match self {
            DevcordEvent::UserEvent(event_type) => event_type.event_type(),
            DevcordEvent::AuthEvent(event_type) => event_type.event_type(),
            DevcordEvent::MessageEvent(event_type) => event_type.event_type(),
            DevcordEvent::GroupEvent(event_type) => event_type.event_type(),
        }
    }
}
