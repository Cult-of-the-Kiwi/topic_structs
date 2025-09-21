use fluvio::RecordKey;
use serde::{Deserialize, Serialize};

use crate::publisher::topic::{TopicEvent, fluvio::KeyEvent};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserCreated {
    pub id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserLoggedIn {
    pub id: String,
    pub username: String,
    pub login_time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserLoggedOut {
    pub id: String,
    pub logout_time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum AuthEvent {
    #[serde(rename = "user_created")]
    UserCreatedEvent(UserCreated),
    #[serde(rename = "user_logged_in")]
    UserLoggedInEvent(UserLoggedIn),
    #[serde(rename = "user_logged_out")]
    UserLoggedOutEvent(UserLoggedOut),
}

impl TopicEvent for AuthEvent {
    fn event_topic(&self) -> crate::publisher::topic::Topic {
        match self {
            AuthEvent::UserCreatedEvent(_) => String::from("auth-user"),
            AuthEvent::UserLoggedInEvent(_) => String::from("auth"),
            AuthEvent::UserLoggedOutEvent(_) => String::from("auth"),
        }
    }
}

impl KeyEvent for AuthEvent {
    fn event_key(&self) -> fluvio::RecordKey {
        match self {
            AuthEvent::UserCreatedEvent(_) => RecordKey::NULL,
            AuthEvent::UserLoggedInEvent(_) => RecordKey::NULL,
            AuthEvent::UserLoggedOutEvent(_) => RecordKey::NULL,
        }
    }
}
