extern crate failure;
extern crate hey_listen;
extern crate parking_lot;

use hey_listen::EventDispatcher;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    EventType,
}

struct EventListener {
    used_method: bool,
}

impl EventListener {
    fn test_method(&mut self, _event: &Event) {
        self.used_method = true;
    }
}

#[test]
fn test_closure() {
    let listener = Arc::new(Mutex::new(EventListener { used_method: false }));
    let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));

    let closure = Box::new(move |event: &Event| {
        let listener = weak_listener_ref.upgrade().unwrap();
        listener.lock().test_method(event);

        None
    });

    let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
    dispatcher.add_fn(Event::EventType, closure);
    dispatcher.dispatch_event(&Event::EventType);

    let listener = listener.lock();
    assert!(listener.used_method);
}
