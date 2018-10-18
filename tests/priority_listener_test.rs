extern crate hey_listen;

use hey_listen::{Listener, Mutex, PriorityEventDispatcher, SyncDispatcherRequest};
use std::sync::Arc;

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
    dispatcher.add_listener(Event::EventType, &last_receiver_a, 3);
    dispatcher.add_listener(Event::EventType, &last_receiver_b, 3);
    dispatcher.add_listener(Event::EventType, &second_receiver_a, 2);
    dispatcher.add_listener(Event::EventType, &first_receiver_a, 1);
    dispatcher.add_listener(Event::EventType, &first_receiver_b, 1);
    dispatcher.add_listener(Event::EventType, &second_receiver_b, 2);

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
    dispatcher.add_listener(Event::EventType, &receiver, 0);

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

    dispatcher.add_listener(Event::EventType, &receiver_a, 0);
    dispatcher.add_listener(Event::EventType, &receiver_b, 0);

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

    dispatcher.add_listener(Event::EventType, &receiver_a, 0);
    dispatcher.add_listener(Event::EventType, &receiver_b, 0);

    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);

    assert_eq!(receiver_a.try_lock().unwrap().times_dispatched, 1);
    assert_eq!(receiver_b.try_lock().unwrap().times_dispatched, 1);
}

#[test]
fn stop_listening_of_fns() {
    let counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter));
    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();

    let closure = Box::new(move |_: &Event| -> Option<SyncDispatcherRequest> {
        let counter_ref = weak_counter_ref.upgrade().unwrap();
        *counter_ref.try_lock().unwrap() += 1;

        Some(SyncDispatcherRequest::StopListening)
    });

    dispatcher.add_fn(Event::EventType, closure, 0);
    assert_eq!(*counter.try_lock().unwrap(), 0);

    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(*counter.try_lock().unwrap(), 1);

    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(*counter.try_lock().unwrap(), 1);
}

#[test]
fn stop_propagation_of_fns() {
    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();
    let counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter));
    let first_closure = Box::new(move |_: &Event| -> Option<SyncDispatcherRequest> {
        let counter_ref = &weak_counter_ref.upgrade().unwrap();
        *counter_ref.try_lock().unwrap() += 1;

        Some(SyncDispatcherRequest::StopPropagation)
    });

    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter));
    let second_closure = Box::new(move |_: &Event| -> Option<SyncDispatcherRequest> {
        let counter_ref = &weak_counter_ref.upgrade().unwrap();
        *counter_ref.try_lock().unwrap() += 1;

        Some(SyncDispatcherRequest::StopPropagation)
    });

    dispatcher.add_fn(Event::EventType, first_closure, 0);
    dispatcher.add_fn(Event::EventType, second_closure, 1);
    assert_eq!(*counter.try_lock().unwrap(), 0);

    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(*counter.try_lock().unwrap(), 1);

    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(*counter.try_lock().unwrap(), 2);
}

#[test]
fn stop_listening_and_propagation_of_fns() {
    #[derive(Debug, PartialEq)]
    pub enum ClosureVisitor {
        First,
        Second,
    }

    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::default();
    let visitor_record: Arc<Mutex<Vec<ClosureVisitor>>> = Arc::new(Mutex::new(Vec::new()));

    let weak_record_ref = Arc::downgrade(&Arc::clone(&visitor_record));
    let first_closure = Box::new(move |_: &Event| -> Option<SyncDispatcherRequest> {
        let weak_ref = &weak_record_ref.upgrade().unwrap();
        weak_ref.try_lock().unwrap().push(ClosureVisitor::First);

        Some(SyncDispatcherRequest::StopListeningAndPropagation)
    });

    let weak_record_ref = Arc::downgrade(&Arc::clone(&visitor_record));
    let second_closure = Box::new(move |_: &Event| -> Option<SyncDispatcherRequest> {
        let weak_ref = weak_record_ref.upgrade().unwrap();
        weak_ref.try_lock().unwrap().push(ClosureVisitor::Second);

        Some(SyncDispatcherRequest::StopListeningAndPropagation)
    });

    dispatcher.add_fn(Event::EventType, first_closure, 0);
    dispatcher.add_fn(Event::EventType, second_closure, 1);
    assert!(visitor_record.try_lock().unwrap().is_empty());

    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(*visitor_record.try_lock().unwrap(), [ClosureVisitor::First]);

    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(
        *visitor_record.try_lock().unwrap(),
        [ClosureVisitor::First, ClosureVisitor::Second]
    );
}

#[test]
fn is_send_and_sync() {
    fn assert_send<T: Send + Sync>(_: &T) {};
    assert_send(&PriorityEventDispatcher::<u32, Event>::default());
}
