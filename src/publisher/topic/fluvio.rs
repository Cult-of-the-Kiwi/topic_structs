use fluvio::{
    Fluvio, FluvioClusterConfig, Offset, RecordKey, TopicProducer,
    consumer::ConsumerConfigExtBuilder, metadata::topic::TopicSpec, spu::SpuSocketPool,
};
use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
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

const CONSUMER_OFFSET: &str = "consumer-auto";

pub struct FluvioHandler<T: TypedEvent> {
    fluvio: Fluvio,
    subscribers: SubscriberMap<T>,
    receivers: Arc<RwLock<HashMap<Topic, (usize, JoinHandle<()>)>>>,
    producers: RefCell<ProducerMap>,
}

impl<T: TypedEvent> FluvioHandler<T> {
    pub fn new(fluvio_addr: Option<&str>) -> anyhow::Result<Self> {
        Ok(Self {
            fluvio: if let Some(addr) = fluvio_addr {
                Fluvio::connect_with_config(&FluvioClusterConfig::new(addr)).block_on()?
            } else {
                Fluvio::connect().block_on()?
            },
            subscribers: Default::default(),
            receivers: Default::default(),
            producers: Default::default(),
        })
    }

    pub fn new_with_config(config: &FluvioClusterConfig) -> anyhow::Result<Self> {
        Ok(Self {
            fluvio: Fluvio::connect_with_config(&config).block_on()?,
            subscribers: Default::default(),
            receivers: Default::default(),
            producers: Default::default(),
        })
    }

    pub fn local() -> anyhow::Result<Self> {
        Ok(Self {
            fluvio: Fluvio::connect().block_on()?,
            subscribers: Default::default(),
            receivers: Default::default(),
            producers: Default::default(),
        })
    }
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
        + Serialize
        + Send
        + Sync
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
            .or_insert(new_topic_reader::<T>(
                &event,
                &self.subscribers,
                &self.fluvio,
            )?)
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
        let producer = binding.entry(event.event_topic()).or_insert({
            try_create_topic(&self.fluvio, &event.event_topic())?;
            self.fluvio.topic_producer(event.event_topic()).block_on()?
        });
        producer
            .send(event.event_key(), to_vec(&event)?)
            .block_on()?;
        Ok(())
    }
}

fn new_topic_reader<T>(
    event: &T,
    subscribers: &SubscriberMap<T>,
    fluvio: &Fluvio,
) -> anyhow::Result<(usize, JoinHandle<()>)>
where
    T: TypedEvent + TopicEvent + for<'a> Deserialize<'a> + 'static + Send,
{
    let subscribers = subscribers.clone();
    let topic = event.event_topic();

    try_create_topic(fluvio, &topic)?;

    //FIXME This should be modificable from the outside
    let consumer_config = ConsumerConfigExtBuilder::default()
        .topic(topic.clone())
        .offset_start(Offset::end())
        .offset_consumer(CONSUMER_OFFSET)
        .build()?;

    let mut consumer_stream = fluvio.consumer_with_config(consumer_config).block_on()?;

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
    Ok((0, handle))
}

fn try_create_topic(fluvio: &Fluvio, topic: &String) -> anyhow::Result<()> {
    let admin = fluvio.admin().block_on();

    let topics = admin.all::<TopicSpec>().block_on()?;
    let topic_names = topics
        .iter()
        .map(|topic| topic.name.clone())
        .collect::<Vec<String>>();

    //FIXME This should be modificable from the outside
    if !topic_names.contains(topic) {
        let topic_spec = TopicSpec::new_computed(1, 1, None);
        admin.create(topic.clone(), false, topic_spec).block_on()?
    }

    Ok(())
}
