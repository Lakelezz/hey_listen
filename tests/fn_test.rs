extern crate hey_listen;
extern crate parking_lot;

use hey_listen::closures::EventDispatcher;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Events {
    EventA,
}

struct TestListener {
    used_method: bool,
}

impl TestListener {
    fn test_method(&mut self, _event: &Events) {
        self.used_method = true;
    }
}

#[test]
fn test_closure() {
    let listener = Arc::new(Mutex::new(TestListener { used_method: false }));
    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));

    let closure = Box::new(move |event: &Events| {
        let listener = weak_listener_ref.upgrade().unwrap();
        listener.lock().test_method(&event);
    });

    let mut dispatcher: EventDispatcher<Events> = EventDispatcher::new();
    dispatcher.add_listener(Events::EventA, closure);
    dispatcher.dispatch_event(&Events::EventA);

    let listener = listener.lock();

    assert!(listener.used_method);
}