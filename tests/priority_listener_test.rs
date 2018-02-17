extern crate hey_listen;
extern crate parking_lot;

use hey_listen::PriorityEventDispatcher;
use hey_listen::Listener;
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
    fn on_event(&mut self, _event: &Event) {
        let mut name_record = self.name_record.try_lock().unwrap();
        name_record.push(self.name.clone());
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

    let mut dispatcher = PriorityEventDispatcher::<u32, Event>::new();

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
