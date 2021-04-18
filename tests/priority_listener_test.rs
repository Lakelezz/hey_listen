use hey_listen::{
    sync::{PriorityListener, PriorityDispatcher, PriorityDispatcherRequest},
    RwLock,
};
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    EventType,
}

struct EventListener {
    name: String,
    name_record: Arc<RwLock<Vec<String>>>,
}

impl PriorityListener<Event> for Arc<RwLock<EventListener>> {
    fn on_event(&self, _event: &Event) -> Option<PriorityDispatcherRequest> {
        let listener = self.try_read().expect("Could not lock self");

        listener
            .name_record
            .try_write()
            .expect("Could not lock name_record")
            .push(listener.name.clone());

        None
    }
}

/// **Intended test-behaviour**: Listeners with different priority-level
/// shall be dispatched in order based on their level, here using `u32`,
/// the lower the earlier.
///
/// **Test**: We will register six listeners, two per priority-level (here from 1 to 3).
/// Every listener owns a reference to a record-book and will insert their name upon receiving
/// an event.
/// Hence dispatching mentioned event-type shall result in a vector with our listener's name
/// ordered by their priority-level.
#[test]
fn listeners_dispatch_in_correct_order() {
    let names_record = Arc::new(RwLock::new(Vec::new()));

    let first_receiver_a = Arc::new(RwLock::new(EventListener {
        name: "1".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let last_receiver_a = Arc::new(RwLock::new(EventListener {
        name: "3".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let second_receiver_a = Arc::new(RwLock::new(EventListener {
        name: "2".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let last_receiver_b = Arc::new(RwLock::new(EventListener {
        name: "3".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let first_receiver_b = Arc::new(RwLock::new(EventListener {
        name: "1".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let second_receiver_b = Arc::new(RwLock::new(EventListener {
        name: "2".to_string(),
        name_record: Arc::clone(&names_record),
    }));

    let mut dispatcher = PriorityDispatcher::<u32, Event>::default();
    dispatcher.add_listener(Event::EventType, Arc::clone(&last_receiver_a), 3);
    dispatcher.add_listener(Event::EventType, Arc::clone(&last_receiver_b), 3);
    dispatcher.add_listener(Event::EventType, Arc::clone(&second_receiver_a), 2);
    dispatcher.add_listener(Event::EventType, Arc::clone(&first_receiver_a), 1);
    dispatcher.add_listener(Event::EventType, Arc::clone(&first_receiver_b), 1);
    dispatcher.add_listener(Event::EventType, Arc::clone(&second_receiver_b), 2);

    dispatcher.dispatch_event(&Event::EventType);
    let names_record = names_record.try_write().unwrap();

    assert_eq!(names_record[0], "1");
    assert_eq!(names_record[1], "1");
    assert_eq!(names_record[2], "2");
    assert_eq!(names_record[3], "2");
    assert_eq!(names_record[4], "3");
    assert_eq!(names_record[5], "3");
}

#[test]
fn stop_listening() {
    #[derive(Default)]
    struct EventListener {
        times_dispatched: usize,
    }

    impl PriorityListener<Event> for Arc<RwLock<EventListener>> {
        fn on_event(&self, _event: &Event) -> Option<PriorityDispatcherRequest> {
            self.write().times_dispatched += 1;

            Some(PriorityDispatcherRequest::StopListening)
        }
    }

    let receiver = Arc::new(RwLock::new(EventListener::default()));
    let mut dispatcher = PriorityDispatcher::<u32, Event>::default();
    dispatcher.add_listener(Event::EventType, Arc::clone(&receiver), 0);

    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver.try_write().unwrap().times_dispatched, 1);
}

#[test]
fn stop_propagation() {
    #[derive(Default)]
    struct EventListener {
        times_dispatched: usize,
    }

    impl PriorityListener<Event> for Arc<RwLock<EventListener>> {
        fn on_event(&self, _event: &Event) -> Option<PriorityDispatcherRequest> {
            self.write().times_dispatched += 1;

            Some(PriorityDispatcherRequest::StopPropagation)
        }
    }

    let receiver_a = Arc::new(RwLock::new(EventListener::default()));
    let receiver_b = Arc::new(RwLock::new(EventListener::default()));
    let mut dispatcher = PriorityDispatcher::<u32, Event>::default();

    dispatcher.add_listener(Event::EventType, Arc::clone(&receiver_a), 0);
    dispatcher.add_listener(Event::EventType, Arc::clone(&receiver_b), 0);

    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver_a.try_write().unwrap().times_dispatched, 1);
    assert_eq!(receiver_b.try_write().unwrap().times_dispatched, 0);
}

#[test]
fn stop_listening_and_propagation() {
    #[derive(Default)]
    struct EventListener {
        times_dispatched: usize,
    }

    impl PriorityListener<Event> for Arc<RwLock<EventListener>> {
        fn on_event(&self, _event: &Event) -> Option<PriorityDispatcherRequest> {
            self.write().times_dispatched += 1;
            Some(PriorityDispatcherRequest::StopListeningAndPropagation)
        }
    }

    let receiver_a = Arc::new(RwLock::new(EventListener::default()));
    let receiver_b = Arc::new(RwLock::new(EventListener::default()));
    let mut dispatcher = PriorityDispatcher::<u32, Event>::default();

    dispatcher.add_listener(Event::EventType, Arc::clone(&receiver_a), 0);
    dispatcher.add_listener(Event::EventType, Arc::clone(&receiver_b), 0);

    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver_a.try_write().unwrap().times_dispatched, 1);
    assert_eq!(receiver_b.try_write().unwrap().times_dispatched, 1);
}

#[test]
fn is_send_and_sync() {
    fn assert_send<T: Send + Sync>(_: &T) {}
    assert_send(&PriorityDispatcher::<u32, Event>::default());
}
