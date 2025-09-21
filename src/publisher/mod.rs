use std::{hash::Hash, pin::Pin};

pub mod topic;

type EventSubscriberHdlrFn<T> =
    Box<dyn (FnMut(T) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + Send + Sync>;

pub trait TypedEvent {
    type EventType: Eq + Hash + Clone + Send + Sync;
    fn event_type(&self) -> Self::EventType;
}

pub trait EventPublisher {
    type Event: TypedEvent;
    fn subscribe(
        &self,
        event: Self::Event,
        listener: EventSubscriberHdlrFn<Self::Event>,
    ) -> anyhow::Result<()>;
    fn unsubscribe(&self, event: Self::Event) -> anyhow::Result<()>;
    fn notify(&self, event: Self::Event) -> anyhow::Result<()>;
}
