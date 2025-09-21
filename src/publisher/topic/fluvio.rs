use fluvio::{
    Fluvio, Offset, RecordKey, TopicProducer, consumer::ConsumerConfigExtBuilder,
    metadata::topic::TopicSpec, spu::SpuSocketPool,
};
use pollster::FutureExt;
use serde::Deserialize;
use serde_json::from_slice;
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};
use tokio_stream::StreamExt;
use tracing::error;

use crate::publisher::{
    EventPublisher, EventSubscriberHdlrFn, TypedEvent,
    topic::{Topic, TopicEvent},
};

type SubscriberMap<T> =
    Arc<RwLock<HashMap<<T as TypedEvent>::EventType, EventSubscriberHdlrFn<T>>>>;
type ProducerMap = HashMap<Topic, TopicProducer<SpuSocketPool>>;

pub struct FluvioHandler<T: TypedEvent> {
    fluvio: Fluvio,
    subscribers: SubscriberMap<T>,
    receivers: Arc<RwLock<HashMap<Topic, (usize, JoinHandle<()>)>>>,
    producers: RefCell<ProducerMap>,
}

pub trait KeyEvent {
    fn event_key(&self) -> RecordKey;
}

impl<T> EventPublisher for FluvioHandler<T>
where
    T: TypedEvent
        + TopicEvent
        + KeyEvent
        + for<'a> Deserialize<'a>
        + Send
        + Sync
        + Into<Vec<u8>>
        + 'static,
{
    type Event = T;

    fn subscribe(
        &self,
        event: Self::Event,
        listener: EventSubscriberHdlrFn<Self::Event>,
    ) -> anyhow::Result<()> {
        let mut lock = self.subscribers.write().block_on();
        lock.insert(event.event_type(), listener);

        let mut lock = self.receivers.write().block_on();
        lock.entry(event.event_topic())
            .or_insert_with(|| new_topic_reader::<T>(&event, &self.subscribers, &self.fluvio))
            .0 += 1;
        Ok(())
    }

    fn unsubscribe(&self, event: Self::Event) -> anyhow::Result<()> {
        let mut lock = self.subscribers.write().block_on();
        lock.remove(&event.event_type());

        let mut lock = self.receivers.write().block_on();
        if let Some(counter) = lock.get_mut(&event.event_topic()) {
            counter.0 -= 1;
            if counter.0 <= 0 {
                counter.1.abort();
                lock.remove(&event.event_topic());
            }
        }
        Ok(())
    }

    fn notify(&self, event: Self::Event) -> anyhow::Result<()> {
        let mut binding = self.producers.borrow_mut();
        let producer = binding
            .entry(event.event_topic())
            .or_insert(self.fluvio.topic_producer(event.event_topic()).block_on()?);
        producer.send(event.event_key(), event).block_on()?;
        Ok(())
    }
}

fn new_topic_reader<T>(
    event: &T,
    self_subscribers: &SubscriberMap<T>,
    self_fluvio: &Fluvio,
) -> (usize, JoinHandle<()>)
where
    T: TypedEvent + TopicEvent + for<'a> Deserialize<'a> + 'static + Send,
{
    let subscribers = self_subscribers.clone();
    let topic = event.event_topic();

    let admin = self_fluvio.admin().block_on();

    let topics = admin
        .all::<TopicSpec>()
        .block_on()
        .expect("Failed to list topics");
    let topic_names = topics
        .iter()
        .map(|topic| topic.name.clone())
        .collect::<Vec<String>>();

    //FIXME This should be modificable from the outside
    if !topic_names.contains(&topic) {
        let topic_spec = TopicSpec::new_computed(1, 1, None);
        admin
            .create(topic.clone(), false, topic_spec)
            .block_on()
            .expect(&format!("Error creating topic: {}", topic)); //CRITICAL(Lamoara)Can only break because of race condition, because of diferent admins at the same time 
    }

    //FIXME This should be modificable from the outside
    let consumer_config = ConsumerConfigExtBuilder::default()
        .topic(topic.clone())
        .offset_start(Offset::beginning())
        .build()
        .expect(&format!("Error creating consumer config for {}", topic));

    let mut consumer_stream = self_fluvio
        .consumer_with_config(consumer_config)
        .block_on()
        .expect(&format!("Error creating consumer for: {}", topic));

    let event_type = event.event_type();
    let handle = tokio::spawn(async move {
        while let Some(Ok(record)) = consumer_stream.next().await {
            let Ok(event) =
                from_slice::<T>(record.value()).map_err(|e| error!("Error parsing event: {}", e))
            else {
                continue;
            };

            let mut map = subscribers.write().await;
            let Some(subscriber) = map.get_mut(&event_type) else {
                break;
            };

            subscriber(event).await;
        }
    });
    (0, handle)
}
