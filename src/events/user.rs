use fluvio::RecordKey;
use serde::{Deserialize, Serialize};

use crate::publisher::topic::{TopicEvent, fluvio::KeyEvent};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserUpdated {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FriendRequestCreated {
    pub from_username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FriendRequestAnswered {
    pub from_username: String,
    pub accepted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum UserEvent {
    #[serde(rename = "user_updated")]
    UserUpdatedEvent(UserUpdated),
    #[serde(rename = "friend_request_created")]
    FriendRequestCreatedEvent(FriendRequestCreated),
    #[serde(rename = "friend_request_answered")]
    FriendRequestAnsweredEvent(FriendRequestAnswered),
}

impl TopicEvent for UserEvent {
    fn event_topic(&self) -> crate::publisher::topic::Topic {
        match self {
            UserEvent::UserUpdatedEvent(_) => String::from("user"),
            UserEvent::FriendRequestCreatedEvent(_) => String::from("user-friendship"),
            UserEvent::FriendRequestAnsweredEvent(_) => String::from("user-friendship"),
        }
    }
}

impl KeyEvent for UserEvent {
    fn event_key(&self) -> fluvio::RecordKey {
        RecordKey::NULL
    }
}
