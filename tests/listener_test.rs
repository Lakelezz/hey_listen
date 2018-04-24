extern crate hey_listen;
extern crate parking_lot;

use hey_listen::{EventDispatcher, Listener, SyncDispatcherRequest};
use std::sync::Arc;
use std::ops::Deref;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    EventA,
    EventB,
}

struct EventListener {
    received_event_a: bool,
    received_event_b: bool,
}

impl Listener<Event> for EventListener {
    fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
        match *event {
            Event::EventA => self.received_event_a = true,
            Event::EventB => self.received_event_b = true,
        }
        None
    }
}

enum EnumListener {
    SomeVariant(bool),
}

impl Listener<Event> for EnumListener {
    fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
        if let Event::EventA = *event {
            match *self {
                EnumListener::SomeVariant(ref mut x) => *x = true,
            }
        }
        None
    }
}

/// **Intended test-behaviour**: When registering one listener for one event,
/// only listened event-variants will be dispatched to the listener.
///
/// **Test**: We will register our listener for one test-variant but will
/// dispatch all two variants.
#[test]
fn dispatch_enum_variant_with_field() {
    let listener = Arc::new(Mutex::new(EnumListener::SomeVariant(false)));
    let mut dispatcher = EventDispatcher::<Event>::default();

    {
        dispatcher.add_listener(Event::EventA, &listener);
    }

    dispatcher.dispatch_event(&Event::EventA);

    let enum_field = match *listener.lock().deref() {
        EnumListener::SomeVariant(x) => x,
    };

    assert!(enum_field);
}

/// **Intended test-behaviour**: When registering one listener for one event,
/// only listened event-variants will be dispatched to the listener.
///
/// **Test**: We will register our listener for one test-variant but will
/// dispatch all two variants.
#[test]
fn register_one_enum_listener_for_one_event_variant_but_dispatch_two_variants() {
    let listener = Arc::new(Mutex::new(EventListener {
        received_event_a: false,
        received_event_b: false,
    }));
    let mut dispatcher = EventDispatcher::<Event>::default();

    {
        dispatcher.add_listener(Event::EventA, &listener);
    }

    dispatcher.dispatch_event(&Event::EventA);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Event::EventB);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);
}

/// **Intended test-behaviour**: When registering one listener for two Event,
/// both of them should be dispatched to the listener.
///
/// **Test**: We will register our listener for two variants and will
/// dispatch both variants.
#[test]
fn register_one_listener_for_two_event_variants_and_dispatch_two_variants() {
    let listener = Arc::new(Mutex::new(EventListener {
        received_event_a: false,
        received_event_b: false,
    }));

    let mut dispatcher = EventDispatcher::<Event>::default();

    {
        dispatcher.add_listener(Event::EventA, &listener);
        dispatcher.add_listener(Event::EventB, &listener);
    }

    dispatcher.dispatch_event(&Event::EventA);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Event::EventB);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(b_has_been_received);
}

#[test]
fn register_and_request_stop_listening() {
    #[derive(Clone, Eq, Hash, PartialEq)]
    enum Event {
        EventType,
    }

    struct ListenerStruct {
        dispatched_events: usize,
    }

    impl Listener<Event> for ListenerStruct {
        fn on_event(&mut self, _: &Event) -> Option<SyncDispatcherRequest> {
            self.dispatched_events += 1;
            Some(SyncDispatcherRequest::StopListening)
        }
    }

    let listener = Arc::new(Mutex::new(ListenerStruct {
        dispatched_events: 0,
    }));
    let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();

    dispatcher.add_listener(Event::EventType, &listener);
    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(listener.lock().dispatched_events, 1);
}

#[test]
fn register_one_listener_for_one_event_variant_but_dispatch_two_variants() {
    use std::hash::{Hash, Hasher};
    use std::mem::discriminant;

    #[derive(Clone, Eq)]
    enum Event {
        EventA(i32),
        EventB(i32),
    }

    impl Hash for Event {
        fn hash<H: Hasher>(&self, _state: &mut H) {}
    }

    impl PartialEq for Event {
        fn eq(&self, other: &Event) -> bool {
            discriminant(self) == discriminant(other)
        }
    }

    struct EventListener {
        received_event_a: bool,
        received_event_b: bool,
    }

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
            match *event {
                Event::EventA(_) => self.received_event_a = true,
                Event::EventB(_) => self.received_event_b = true,
            }
            None
        }
    }

    let listener = Arc::new(Mutex::new(EventListener {
        received_event_a: false,
        received_event_b: false,
    }));
    let mut dispatcher = EventDispatcher::<Event>::default();

    {
        dispatcher.add_listener(Event::EventA(5), &listener);
        dispatcher.add_listener(Event::EventB(0), &listener);
    }

    dispatcher.dispatch_event(&Event::EventA(10));
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Event::EventB(10));
    let b_has_been_received = listener.lock().received_event_b;
    assert!(b_has_been_received);
}

#[test]
fn stop_propagation_on_sync_dispatcher() {
    struct EventListener {
        has_been_dispatched: bool,
    }

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, _: &Event) -> Option<SyncDispatcherRequest> {
            self.has_been_dispatched = true;

            Some(SyncDispatcherRequest::StopPropagation)
        }
    }

    let listener_a = Arc::new(Mutex::new(EventListener {
        has_been_dispatched: false,
    }));

    let listener_b = Arc::new(Mutex::new(EventListener {
        has_been_dispatched: false,
    }));

    let mut dispatcher = EventDispatcher::<Event>::default();

    {
        dispatcher.add_listener(Event::EventA, &listener_a);
        dispatcher.add_listener(Event::EventA, &listener_b);
    }

    dispatcher.dispatch_event(&Event::EventA);
    let a_has_been_dispatched = listener_a.try_lock().unwrap().has_been_dispatched;
    let b_has_been_dispatched = listener_b.try_lock().unwrap().has_been_dispatched;
    assert!(a_has_been_dispatched);
    assert!(!b_has_been_dispatched);
}

#[test]
fn stop_listening_and_propagation_on_sync_dispatcher() {
    struct EventListener {
        dispatch_counter: usize,
    }

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, _: &Event) -> Option<SyncDispatcherRequest> {
            self.dispatch_counter += 1;

            Some(SyncDispatcherRequest::StopListeningAndPropagation)
        }
    }

    let listener_a = Arc::new(Mutex::new(EventListener {
        dispatch_counter: 0,
    }));

    let listener_b = Arc::new(Mutex::new(EventListener {
        dispatch_counter: 0,
    }));

    let mut dispatcher = EventDispatcher::<Event>::default();

    {
        dispatcher.add_listener(Event::EventA, &listener_a);
        dispatcher.add_listener(Event::EventA, &listener_b);
    }

    {
        let counter_a = listener_a.try_lock().unwrap().dispatch_counter;
        let counter_b = listener_b.try_lock().unwrap().dispatch_counter;
        assert_eq!(counter_a, 0);
        assert_eq!(counter_b, 0);
    }

    dispatcher.dispatch_event(&Event::EventA);
    {
        let counter_a = listener_a.try_lock().unwrap().dispatch_counter;
        let counter_b = listener_b.try_lock().unwrap().dispatch_counter;
        assert_eq!(counter_a, 1);
        assert_eq!(counter_b, 0);
    }

    dispatcher.dispatch_event(&Event::EventA);
    {
        let counter_a = listener_a.try_lock().unwrap().dispatch_counter;
        let counter_b = listener_b.try_lock().unwrap().dispatch_counter;
        assert_eq!(counter_a, 1);
        assert_eq!(counter_b, 1);
    }

    dispatcher.dispatch_event(&Event::EventA);
    {
        let counter_a = listener_a.try_lock().unwrap().dispatch_counter;
        let counter_b = listener_b.try_lock().unwrap().dispatch_counter;
        assert_eq!(counter_a, 1);
        assert_eq!(counter_b, 1);
    }
}

#[test]
fn stop_listening_on_sync_dispatcher_of_fns() {
    struct EventListener {
        use_counter: usize,
    };

    let listener = Arc::new(Mutex::new(EventListener { use_counter: 0 }));

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.lock().use_counter += 1;

        Some(SyncDispatcherRequest::StopListening)
    });

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.lock().use_counter += 1;

        Some(SyncDispatcherRequest::StopListening)
    });

    {
        let counter = listener.try_lock().unwrap().use_counter;
        assert_eq!(counter, 0);
    }

    let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
    dispatcher.add_fn(Event::EventA, closure_a);
    dispatcher.add_fn(Event::EventA, closure_b);
    dispatcher.dispatch_event(&Event::EventA);

    {
        let counter = listener.try_lock().unwrap().use_counter;
        assert_eq!(counter, 2);
    }

    {
        dispatcher.dispatch_event(&Event::EventA);
        let counter = listener.try_lock().unwrap().use_counter;
        assert_eq!(counter, 2);
    }
}

#[test]
fn stop_propagation_on_sync_dispatcher_of_fns() {
    struct EventListener {
        use_counter: usize,
    };

    let listener = Arc::new(Mutex::new(EventListener { use_counter: 0 }));

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.lock().use_counter += 1;

        Some(SyncDispatcherRequest::StopPropagation)
    });

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.lock().use_counter += 1;

        Some(SyncDispatcherRequest::StopPropagation)
    });

    {
        let counter = listener.try_lock().unwrap().use_counter;
        assert_eq!(counter, 0);
    }

    let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
    dispatcher.add_fn(Event::EventA, closure_a);
    dispatcher.add_fn(Event::EventA, closure_b);
    dispatcher.dispatch_event(&Event::EventA);

    {
        let counter = listener.try_lock().unwrap().use_counter;
        assert_eq!(counter, 1);
    }

    {
        dispatcher.dispatch_event(&Event::EventA);
        let counter = listener.try_lock().unwrap().use_counter;
        assert_eq!(counter, 2);
    }
}
