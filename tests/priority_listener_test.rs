extern crate hey_listen;
extern crate parking_lot;

use hey_listen::{Listener, PriorityEventDispatcher, SyncDispatcherRequest};
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    EventType,
}

struct EventListener {
    name: String,
    name_record: Arc<Mutex<Vec<String>>>,
}

impl Listener<Event> for EventListener {
    fn on_event(&mut self, _event: &Event) -> Option<SyncDispatcherRequest> {
        let mut name_record = self.name_record.try_lock().unwrap();
        name_record.push(self.name.clone());
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
    let names_record = Arc::new(Mutex::new(Vec::new()));

    let first_receiver_a = Arc::new(Mutex::new(EventListener {
        name: "1".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let last_receiver_a = Arc::new(Mutex::new(EventListener {
        name: "3".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let second_receiver_a = Arc::new(Mutex::new(EventListener {
        name: "2".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let last_receiver_b = Arc::new(Mutex::new(EventListener {
        name: "3".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let first_receiver_b = Arc::new(Mutex::new(EventListener {
        name: "1".to_string(),
        name_record: Arc::clone(&names_record),
    }));
    let second_receiver_b = Arc::new(Mutex::new(EventListener {
        name: "2".to_string(),
        name_record: Arc::clone(&names_record),
    }));

    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();

    {
        dispatcher.add_listener(Event::EventType, &last_receiver_a, 3);
        dispatcher.add_listener(Event::EventType, &last_receiver_b, 3);
        dispatcher.add_listener(Event::EventType, &second_receiver_a, 2);
        dispatcher.add_listener(Event::EventType, &first_receiver_a, 1);
        dispatcher.add_listener(Event::EventType, &first_receiver_b, 1);
        dispatcher.add_listener(Event::EventType, &second_receiver_b, 2);
    }

    dispatcher.dispatch_event(&Event::EventType);

    let names_record = names_record.try_lock().unwrap();

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

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, _event: &Event) -> Option<SyncDispatcherRequest> {
            self.times_dispatched += 1;
            Some(SyncDispatcherRequest::StopListening)
        }
    }

    let receiver = Arc::new(Mutex::new(EventListener::default()));
    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();

    {
        dispatcher.add_listener(Event::EventType, &receiver, 0);
    }

    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver.try_lock().unwrap().times_dispatched, 1);
}

#[test]
fn stop_propagation() {
    #[derive(Default)]
    struct EventListener {
        times_dispatched: usize,
    }

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, _event: &Event) -> Option<SyncDispatcherRequest> {
            self.times_dispatched += 1;
            Some(SyncDispatcherRequest::StopPropagation)
        }
    }

    let receiver_a = Arc::new(Mutex::new(EventListener::default()));
    let receiver_b = Arc::new(Mutex::new(EventListener::default()));
    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();

    {
        dispatcher.add_listener(Event::EventType, &receiver_a, 0);
        dispatcher.add_listener(Event::EventType, &receiver_b, 0);
    }

    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver_a.try_lock().unwrap().times_dispatched, 1);
    assert_eq!(receiver_b.try_lock().unwrap().times_dispatched, 0);
}

#[test]
fn stop_listening_and_propagation() {
    #[derive(Default)]
    struct EventListener {
        times_dispatched: usize,
    }

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, _event: &Event) -> Option<SyncDispatcherRequest> {
            self.times_dispatched += 1;
            Some(SyncDispatcherRequest::StopListeningAndPropagation)
        }
    }

    let receiver_a = Arc::new(Mutex::new(EventListener::default()));
    let receiver_b = Arc::new(Mutex::new(EventListener::default()));
    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();

    {
        dispatcher.add_listener(Event::EventType, &receiver_a, 0);
        dispatcher.add_listener(Event::EventType, &receiver_b, 0);
    }

    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver_a.try_lock().unwrap().times_dispatched, 1);
    assert_eq!(receiver_b.try_lock().unwrap().times_dispatched, 1);
}
