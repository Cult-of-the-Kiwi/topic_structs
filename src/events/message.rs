use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MessageSent {
    pub channel_id: String,
    pub sender: String,
    pub message: String,
}
