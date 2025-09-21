use fluvio::RecordKey;
use serde::{Deserialize, Serialize};

use crate::publisher::topic::{TopicEvent, fluvio::KeyEvent};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GroupCreatedEvent {
    pub group_id: String,
    pub owner_id: String,
    pub channel_id: String,
    pub member_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GroupDeletedEvent {
    pub group_id: String,
    pub owner_id: String,
    pub member_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GroupUserAddedEvent {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GroupUserRemovedEvent {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum GroupEvent {
    #[serde(rename = "group_created")]
    GroupCreatedEvent(GroupCreatedEvent),
    #[serde(rename = "group_deleted")]
    GroupDeletedEvent(GroupDeletedEvent),
    #[serde(rename = "group_user_added")]
    GroupUserAddedEvent(GroupUserAddedEvent),
    #[serde(rename = "group_user_removed")]
    GroupUserRemovedEvent(GroupUserRemovedEvent),
}

impl TopicEvent for GroupEvent {
    fn event_topic(&self) -> crate::publisher::topic::Topic {
        match self {
            GroupEvent::GroupCreatedEvent(_) => String::from("group"),
            GroupEvent::GroupDeletedEvent(_) => String::from("group"),
            GroupEvent::GroupUserAddedEvent(_) => String::from("group"),
            GroupEvent::GroupUserRemovedEvent(_) => String::from("group"),
        }
    }
}

impl KeyEvent for GroupEvent {
    fn event_key(&self) -> fluvio::RecordKey {
        match self {
            GroupEvent::GroupCreatedEvent(_) => RecordKey::NULL,
            GroupEvent::GroupDeletedEvent(_) => RecordKey::NULL,
            GroupEvent::GroupUserAddedEvent(_) => RecordKey::NULL,
            GroupEvent::GroupUserRemovedEvent(_) => RecordKey::NULL,
        }
    }
}
