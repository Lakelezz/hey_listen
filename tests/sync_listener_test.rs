use hey_listen::{
    sync::{Dispatcher, Listener, SyncDispatcherRequest},
    RwLock,
};
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    VariantA,
    VariantB,
}

struct EventListener {
    received_variant_a: bool,
    received_variant_b: bool,
}

impl Listener<Event> for EventListener {
    fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
        match *event {
            Event::VariantA => self.received_variant_a = true,
            Event::VariantB => self.received_variant_b = true,
        }
        None
    }
}

enum EnumListener {
    SomeVariant(bool),
}

impl Listener<Event> for EnumListener {
    fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
        if let Event::VariantA = *event {
            match *self {
                EnumListener::SomeVariant(ref mut x) => *x = true,
            }
        }
        None
    }
}

#[test]
fn dispatch_enum_variant_with_field() {
    let listener = Arc::new(RwLock::new(EnumListener::SomeVariant(false)));
    let mut dispatcher = Dispatcher::<Event>::default();
    dispatcher.add_listener(Event::VariantA, &listener);

    dispatcher.dispatch_event(&Event::VariantA);

    let enum_field = match *listener.write().deref() {
        EnumListener::SomeVariant(x) => x,
    };

    assert!(enum_field);
}

#[test]
fn register_one_enum_listener_for_one_event_variant_but_dispatch_two_variants() {
    let listener = Arc::new(RwLock::new(EventListener {
        received_variant_a: false,
        received_variant_b: false,
    }));

    let mut dispatcher = Dispatcher::<Event>::default();
    dispatcher.add_listener(Event::VariantA, &listener);

    dispatcher.dispatch_event(&Event::VariantA);
    let a_has_been_received = listener.try_write().unwrap().received_variant_a;
    let b_has_been_received = listener.try_write().unwrap().received_variant_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Event::VariantB);
    let a_has_been_received = listener.try_write().unwrap().received_variant_a;
    let b_has_been_received = listener.try_write().unwrap().received_variant_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);
}

#[test]
fn register_one_listener_for_two_event_variants_and_dispatch_two_variants() {
    let listener = Arc::new(RwLock::new(EventListener {
        received_variant_a: false,
        received_variant_b: false,
    }));

    let mut dispatcher = Dispatcher::<Event>::default();

    dispatcher.add_listener(Event::VariantA, &listener);
    dispatcher.add_listener(Event::VariantB, &listener);

    dispatcher.dispatch_event(&Event::VariantA);
    let a_has_been_received = listener.write().received_variant_a;
    let b_has_been_received = listener.write().received_variant_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Event::VariantB);
    let a_has_been_received = listener.write().received_variant_a;
    let b_has_been_received = listener.write().received_variant_b;
    assert!(a_has_been_received);
    assert!(b_has_been_received);
}

#[test]
fn dispatch_to_function() {
    struct EventListener {
        used_method: bool,
    }

    impl EventListener {
        fn test_method(&mut self, _event: &Event) {
            self.used_method = true;
        }
    }

    let listener = Arc::new(RwLock::new(EventListener { used_method: false }));
    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));

    let closure = Box::new(move |event: &Event| {
        let listener = weak_listener_ref.upgrade().unwrap();
        listener.write().test_method(event);

        None
    });

    let mut dispatcher: Dispatcher<Event> = Dispatcher::default();
    dispatcher.add_fn(Event::VariantA, closure);
    dispatcher.dispatch_event(&Event::VariantA);

    let listener = listener.write();
    assert!(listener.used_method);
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

    let listener = Arc::new(RwLock::new(ListenerStruct {
        dispatched_events: 0,
    }));

    let mut dispatcher: Dispatcher<Event> = Dispatcher::default();

    dispatcher.add_listener(Event::EventType, &listener);
    dispatcher.dispatch_event(&Event::EventType);
    dispatcher.dispatch_event(&Event::EventType);
    assert_eq!(listener.write().dispatched_events, 1);
}

#[test]
fn register_one_listener_for_one_event_variant_but_dispatch_two_variants() {
    use std::hash::{Hash, Hasher};
    use std::mem::discriminant;

    #[derive(Clone, Eq)]
    enum Event {
        VariantA(i32),
        VariantB(i32),
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
        received_variant_a: bool,
        received_variant_b: bool,
    }

    impl Listener<Event> for EventListener {
        fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
            match *event {
                Event::VariantA(_) => self.received_variant_a = true,
                Event::VariantB(_) => self.received_variant_b = true,
            }
            None
        }
    }

    let listener = Arc::new(RwLock::new(EventListener {
        received_variant_a: false,
        received_variant_b: false,
    }));
    let mut dispatcher = Dispatcher::<Event>::default();

    dispatcher.add_listener(Event::VariantA(5), &listener);
    dispatcher.add_listener(Event::VariantB(0), &listener);

    dispatcher.dispatch_event(&Event::VariantA(10));
    let a_has_been_received = listener.write().received_variant_a;
    let b_has_been_received = listener.write().received_variant_b;
    assert!(a_has_been_received);
    assert!(!b_has_been_received);

    dispatcher.dispatch_event(&Event::VariantB(10));
    let b_has_been_received = listener.write().received_variant_b;
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

    let listener_a = Arc::new(RwLock::new(EventListener {
        has_been_dispatched: false,
    }));

    let listener_b = Arc::new(RwLock::new(EventListener {
        has_been_dispatched: false,
    }));

    let mut dispatcher = Dispatcher::<Event>::default();

    dispatcher.add_listener(Event::VariantA, &listener_a);
    dispatcher.add_listener(Event::VariantA, &listener_b);

    dispatcher.dispatch_event(&Event::VariantA);
    let a_has_been_dispatched = listener_a.try_write().unwrap().has_been_dispatched;
    let b_has_been_dispatched = listener_b.try_write().unwrap().has_been_dispatched;
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

    let listener_a = Arc::new(RwLock::new(EventListener {
        dispatch_counter: 0,
    }));

    let listener_b = Arc::new(RwLock::new(EventListener {
        dispatch_counter: 0,
    }));

    let mut dispatcher = Dispatcher::<Event>::default();

    dispatcher.add_listener(Event::VariantA, &listener_a);
    dispatcher.add_listener(Event::VariantA, &listener_b);

    let counter_a = listener_a.try_write().unwrap().dispatch_counter;
    let counter_b = listener_b.try_write().unwrap().dispatch_counter;
    assert_eq!(counter_a, 0);
    assert_eq!(counter_b, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter_a = listener_a.try_write().unwrap().dispatch_counter;
    let counter_b = listener_b.try_write().unwrap().dispatch_counter;
    assert_eq!(counter_a, 1);
    assert_eq!(counter_b, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter_a = listener_a.try_write().unwrap().dispatch_counter;
    let counter_b = listener_b.try_write().unwrap().dispatch_counter;
    assert_eq!(counter_a, 1);
    assert_eq!(counter_b, 1);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter_a = listener_a.try_write().unwrap().dispatch_counter;
    let counter_b = listener_b.try_write().unwrap().dispatch_counter;
    assert_eq!(counter_a, 1);
    assert_eq!(counter_b, 1);
}

#[test]
fn stop_listening_on_sync_dispatcher_of_fns() {
    struct EventListener {
        use_counter: usize,
    };

    let listener = Arc::new(RwLock::new(EventListener { use_counter: 0 }));

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.write().use_counter += 1;

        Some(SyncDispatcherRequest::StopListening)
    });

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.write().use_counter += 1;

        Some(SyncDispatcherRequest::StopListening)
    });

    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 0);

    let mut dispatcher: Dispatcher<Event> = Dispatcher::default();
    dispatcher.add_fn(Event::VariantA, closure_a);
    dispatcher.add_fn(Event::VariantA, closure_b);
    dispatcher.dispatch_event(&Event::VariantA);

    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 2);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 2);
}

#[test]
fn stop_propagation_on_sync_dispatcher_of_fns() {
    struct EventListener {
        use_counter: usize,
    };

    let listener = Arc::new(RwLock::new(EventListener { use_counter: 0 }));

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.write().use_counter += 1;

        Some(SyncDispatcherRequest::StopPropagation)
    });

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.write().use_counter += 1;

        Some(SyncDispatcherRequest::StopPropagation)
    });

    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 0);

    let mut dispatcher: Dispatcher<Event> = Dispatcher::default();
    dispatcher.add_fn(Event::VariantA, closure_a);
    dispatcher.add_fn(Event::VariantA, closure_b);
    dispatcher.dispatch_event(&Event::VariantA);

    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 1);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 2);
}

#[test]
fn stop_propagation_and_listening_on_sync_dispatcher_of_fns() {
    struct EventListener {
        use_counter: usize,
    };

    let listener = Arc::new(RwLock::new(EventListener { use_counter: 0 }));

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.write().use_counter += 1;

        Some(SyncDispatcherRequest::StopListeningAndPropagation)
    });

    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_listener_ref.upgrade().unwrap();
        listener.write().use_counter += 1;

        Some(SyncDispatcherRequest::StopListeningAndPropagation)
    });

    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 0);

    let mut dispatcher: Dispatcher<Event> = Dispatcher::default();
    dispatcher.add_fn(Event::VariantA, closure_a);
    dispatcher.add_fn(Event::VariantA, closure_b);
    dispatcher.dispatch_event(&Event::VariantA);

    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 1);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 2);

    dispatcher.dispatch_event(&Event::VariantA);
    let counter = listener.try_write().unwrap().use_counter;
    assert_eq!(counter, 2);
}

#[test]
fn is_send_and_sync() {
    fn assert_send<T: Send + Sync>(_: &T) {};
    assert_send(&Dispatcher::<Event>::default());
}
