use serde::{Deserialize, Serialize};

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
