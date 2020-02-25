use hey_listen::{
    sync::{ParallelDispatcher, ParallelDispatcherRequest, ParallelListener},
    Mutex, RwLock,
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

    impl ParallelListener<Event> for Mutex<CountingEventListener> {
        fn on_event(&self, _event: &Event) -> Option<ParallelDispatcherRequest> {
            self.lock().dispatch_counter += 1;

            None
        }
    }

    impl ParallelListener<Event> for Arc<RwLock<CountingEventListener>> {
        fn on_event(&self, _event: &Event) -> Option<ParallelDispatcherRequest> {
            self.write().dispatch_counter += 1;

            None
        }
    }

    let mut dispatcher =
        ParallelDispatcher::<Event>::new(1).expect("Failed constructing threadpool");
    let listener_a = Mutex::new(CountingEventListener::default());
    let listener_b = Arc::new(RwLock::new(CountingEventListener::default()));
    let listener_c = Arc::new(RwLock::new(CountingEventListener::default()));

    dispatcher.add_listener(Event::VariantA, listener_a);
    dispatcher.add_listener(Event::VariantB, Arc::clone(&listener_b));
    dispatcher.add_listener(Event::VariantA, Arc::clone(&listener_c));

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);
    assert_eq!(listener_c.try_write().unwrap().dispatch_counter, 1);

    dispatcher.dispatch_event(&Event::VariantA);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 0);
    assert_eq!(listener_c.try_write().unwrap().dispatch_counter, 2);

    dispatcher.dispatch_event(&Event::VariantA);
    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 1);
    assert_eq!(listener_c.try_write().unwrap().dispatch_counter, 3);

    dispatcher.dispatch_event(&Event::VariantB);
    assert_eq!(listener_b.try_write().unwrap().dispatch_counter, 2);
    assert_eq!(listener_c.try_write().unwrap().dispatch_counter, 3);
}

#[test]
fn is_send_and_sync() {
    fn assert_send<T: Send + Sync>(_: &T) {};
    assert_send(&ParallelDispatcher::<Event>::new(0).unwrap());
}
