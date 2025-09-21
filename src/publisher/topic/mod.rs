pub mod fluvio;

pub type Topic = String;

pub trait TopicEvent {
    fn event_topic(&self) -> Topic;
}
