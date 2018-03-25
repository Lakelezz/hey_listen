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
