pub mod fluvio;

pub type Topic = &'static str;

pub trait TopicEvent {
    fn event_topic(&self) -> Topic;
}
