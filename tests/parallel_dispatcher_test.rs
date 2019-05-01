use hey_listen::{
    sync::{ParallelDispatcherRequest, ParallelDispatcher, ParallelListener},
    RwLock,
};
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    VariantA,
    VariantB,
}

#[test]
fn dispatch_parallel_to_dyn_traits() {
    #[derive(Default)]
    struct CountingEventListener {
        dispatch_counter: usize,
    }

    impl ParallelListener<Event> for CountingEventListener {
        fn on_event(&mut self, _event: &Event) -> Option<ParallelDispatcherRequest> {
            self.dispatch_counter += 1;

            None
        }
    }

    let mut dispatcher = ParallelDispatcher::<Event>::default();
    let listener_a = Arc::new(RwLock::new(CountingEventListener::default()));
    let listener_b = Arc::new(RwLock::new(CountingEventListener::default()));

    dispatcher.add_listener(Event::VariantA, &listener_a);
    dispatcher.add_listener(Event::VariantB, &listener_b);

    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 0);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 1);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 2);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 3);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 1);

    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 3);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 2);
}

#[test]
fn dispatch_parallel_to_functions() {
    let mut dispatcher = ParallelDispatcher::<Event>::default();

    #[derive(Default)]
    struct DispatchCounter {
        counter: usize,
    };

    let counter_a = Arc::new(RwLock::new(DispatchCounter::default()));
    let counter_b = Arc::new(RwLock::new(DispatchCounter::default()));

    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter_a));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_counter_ref.upgrade().unwrap();
        listener.try_write().unwrap().counter += 1;

        None
    });

    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter_b));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_counter_ref.upgrade().unwrap();
        listener.try_write().unwrap().counter += 1;

        None
    });

    dispatcher.add_fn(Event::VariantA, closure_a);
    dispatcher.add_fn(Event::VariantB, closure_b);

    assert_eq!(counter_a.try_write().unwrap().counter, 0);
    assert_eq!(counter_b.try_write().unwrap().counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(counter_a.try_write().unwrap().counter, 1);
    assert_eq!(counter_b.try_write().unwrap().counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(counter_a.try_write().unwrap().counter, 2);
    assert_eq!(counter_b.try_write().unwrap().counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(counter_a.try_write().unwrap().counter, 3);
    assert_eq!(counter_b.try_write().unwrap().counter, 1);

    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(counter_a.try_write().unwrap().counter, 3);
    assert_eq!(counter_b.try_write().unwrap().counter, 2);
}

#[test]
fn stop_listening_parallel_for_dyn_traits() {
    #[derive(Default)]
    struct CountingEventListener {
        dispatch_counter: usize,
    }

    impl ParallelListener<Event> for CountingEventListener {
        fn on_event(&mut self, _event: &Event) -> Option<ParallelDispatcherRequest> {
            self.dispatch_counter += 1;

            Some(ParallelDispatcherRequest::StopListening)
        }
    }

    let mut dispatcher = ParallelDispatcher::<Event>::default();
    let listener_a = Arc::new(RwLock::new(CountingEventListener::default()));
    let listener_b = Arc::new(RwLock::new(CountingEventListener::default()));

    dispatcher.add_listener(Event::VariantA, &listener_a);
    dispatcher.add_listener(Event::VariantB, &listener_b);

    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 0);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 1);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 1);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 1);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 1);

    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(listener_a.try_write().unwrap().dispatch_counter, 1);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 1);
}

#[test]
fn stop_listening_parallel_for_fns() {
    let mut dispatcher = ParallelDispatcher::<Event>::default();

    #[derive(Default)]
    struct DispatchCounter {
        counter: usize,
    };

    let counter_a = Arc::new(RwLock::new(DispatchCounter::default()));
    let counter_b = Arc::new(RwLock::new(DispatchCounter::default()));

    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter_a));
    let closure_a = Box::new(move |_event: &Event| {
        let listener = &weak_counter_ref.upgrade().unwrap();
        listener.try_write().unwrap().counter += 1;

        None
    });

    let weak_counter_ref = Arc::downgrade(&Arc::clone(&counter_b));
    let closure_b = Box::new(move |_event: &Event| {
        let listener = &weak_counter_ref.upgrade().unwrap();
        listener.try_write().unwrap().counter += 1;

        None
    });

    dispatcher.add_fn(Event::VariantA, closure_a);
    dispatcher.add_fn(Event::VariantB, closure_b);

    assert_eq!(counter_a.try_write().unwrap().counter, 0);
    assert_eq!(counter_b.try_write().unwrap().counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(counter_a.try_write().unwrap().counter, 1);
    assert_eq!(counter_b.try_write().unwrap().counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(counter_a.try_write().unwrap().counter, 2);
    assert_eq!(counter_b.try_write().unwrap().counter, 0);

    dispatcher.dispatch_event(&Event::VariantA);
    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(counter_a.try_write().unwrap().counter, 3);
    assert_eq!(counter_b.try_write().unwrap().counter, 1);

    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(counter_a.try_write().unwrap().counter, 3);
    assert_eq!(counter_b.try_write().unwrap().counter, 2);
}

#[test]
fn is_send_and_sync() {
    fn assert_send<T: Send + Sync>(_: &T) {};
    assert_send(&ParallelDispatcher::<Event>::default());
}
