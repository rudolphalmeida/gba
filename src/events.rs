use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

pub type EventId = &'static str;

pub struct Payload {
    value: Box<dyn Any>,
}

impl Payload {
    pub fn new(value: Box<dyn Any>) -> Self {
        Self { value }
    }

    pub fn get_ref<T: 'static>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }
}

pub trait Event: Debug + Send + Sync {
    fn event_id(&self) -> EventId;
    fn payload(&self) -> Option<Payload> {
        None
    }
}

pub type HandlerCallback = Arc<dyn Fn(&dyn Event) + Send + Sync>;

pub struct EventDispatcher {
    event_id: EventId,
    handlers: Arc<Mutex<Vec<HandlerCallback>>>,
}

impl EventDispatcher {
    pub fn new(event_id: EventId) -> Self {
        Self {
            event_id,
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_handler(&mut self, handler: HandlerCallback) {
        self.handlers.lock().unwrap().push(handler);
    }

    pub fn dispatch(&self, event: &dyn Event) {
        let event_id = event.event_id();
        if event_id != self.event_id {
            return;
        }

        self.handlers
            .lock()
            .unwrap()
            .iter()
            .for_each(|handler| handler(event));
    }
}

pub struct EventBus {
    dispatchers: Arc<Mutex<HashMap<EventId, EventDispatcher>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            dispatchers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_handler(&mut self, event_id: EventId, handler: HandlerCallback) {
        self.dispatchers
            .lock()
            .unwrap()
            .entry(event_id)
            .or_insert(EventDispatcher::new(event_id))
            .register_handler(handler);
    }

    pub fn dispatch(&self, event: &dyn Event) {
        let event_id = event.event_id();
        if let Some(dispatcher) = self.dispatchers.lock().unwrap().get(event_id) {
            dispatcher.dispatch(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestEvent {
        value: u32,
    }

    const TEST_EVENT_ID: &'static str = "TEST_EVENT";

    impl Event for TestEvent {
        fn event_id(&self) -> EventId {
            TEST_EVENT_ID
        }

        fn payload(&self) -> Option<Payload> {
            Some(Payload::new(Box::new(self.value)))
        }
    }

    fn handler(event: &dyn Event) {
        let value = *event.payload().unwrap().get_ref::<u32>().unwrap();
        assert_eq!(value, 10);
    }

    #[test]
    fn should_dispatch() {
        let mut event_bus = EventBus::new();

        event_bus.register_handler(TEST_EVENT_ID, Arc::new(handler));
        event_bus.register_handler(TEST_EVENT_ID, Arc::new(move |_| {}));

        let test_event = TestEvent { value: 10 };

        event_bus.dispatch(&test_event);
    }
}
