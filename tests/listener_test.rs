extern crate hey_listen;
extern crate parking_lot;

use hey_listen::EventDispatcher;
use hey_listen::Listener;
use std::sync::Arc;
use std::ops::Deref;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Events {
    EventA,
    EventB,
}

struct TestListener {
    received_event_a: bool,
    received_event_b: bool,
}

impl Listener<Events> for TestListener {
    fn on_event(&mut self, event: &Events) {
        match *event {
            Events::EventA => self.received_event_a = true,
            Events::EventB => self.received_event_b = true,
        }
    }
}

enum EnumListener {
    SomeVariant(bool)
}

impl Listener<Events> for EnumListener {
    fn on_event(&mut self, event: &Events) {
        if let Events::EventA = *event {
            match *self {
                EnumListener::SomeVariant(ref mut x) => *x = true,
            }
        }
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
    let mut dispatcher = EventDispatcher::<Events>::new();

    {
        dispatcher.add_listener(Events::EventA, &listener);
    }

    dispatcher.dispatch_event(&Events::EventA);

    let enum_field = match *listener.lock().deref() {
        EnumListener::SomeVariant(x) => x
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
    let listener = Arc::new(Mutex::new(TestListener { received_event_a: false, received_event_b: false }));
    let mut dispatcher = EventDispatcher::<Events>::new();

    {
        dispatcher.add_listener(Events::EventA, &listener);
    }

    dispatcher.dispatch_event(&Events::EventA);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Events::EventB);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);
}

/// **Intended test-behaviour**: When registering one listener for two events,
/// both of them should be dispatched to the listener.
///
/// **Test**: We will register our listener for two variants and will
/// dispatch both variants.
#[test]
fn register_one_listener_for_two_event_variants_and_dispatch_two_variants() {
    let listener = Arc::new(Mutex::new(TestListener { received_event_a: false, received_event_b: false }));

    let mut dispatcher = EventDispatcher::<Events>::new();

    {
        dispatcher.add_listener(Events::EventA, &listener);
        dispatcher.add_listener(Events::EventB, &listener);
    }

    dispatcher.dispatch_event(&Events::EventA);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Events::EventB);
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(b_has_been_received);
}

#[test]
fn register_one_listener_for_one_event_variant_but_dispatch_two_variants() {
    use std::hash::{Hasher, Hash};
    use std::mem::discriminant;

    #[derive(Clone, Eq)]
    enum Events {
        EventA(i32),
        EventB(i32),
    }

    impl Hash for Events {
        fn hash<H: Hasher>(&self, _state: &mut H) {}
    }

    impl PartialEq for Events {
        fn eq(&self, other: &Events) -> bool {
            discriminant(self) == discriminant(other)
        }
    }

    struct TestListener {
        received_event_a: bool,
        received_event_b: bool,
    }

    impl Listener<Events> for TestListener {
        fn on_event(&mut self, event: &Events) {
            match *event {
                Events::EventA(_) => self.received_event_a = true,
                Events::EventB(_) => self.received_event_b = true,
            }
        }
    }

    let listener = Arc::new(Mutex::new(TestListener { received_event_a: false, received_event_b: false }));
    let mut dispatcher = EventDispatcher::<Events>::new();

    {
        dispatcher.add_listener(Events::EventA(5), &listener);
        dispatcher.add_listener(Events::EventB(0), &listener);
    }

    dispatcher.dispatch_event(&Events::EventA(10));
    let a_has_been_received = listener.lock().received_event_a;
    let b_has_been_received = listener.lock().received_event_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Events::EventB(10));
    let b_has_been_received = listener.lock().received_event_b;
    assert!(b_has_been_received);
}