extern crate hey_listen;
extern crate parking_lot;

use hey_listen::PriorityEventDispatcher;
use hey_listen::Listener;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Events {
    Event,
}

struct TestListener {
    name: String,
    name_record: Arc<Mutex<Vec<String>>>,
}

impl Listener<Events> for TestListener {
    fn on_event(&mut self, _event: &Events) {
        let mut name_record = self.name_record.try_lock().unwrap();
        name_record.push(self.name.clone());
    }
}

/// **Intended test-behaviour**: Listeners with different priority-level
/// shall be dispatched in order based on their level - the higher the
/// earlier.
///
/// **Test**: We will register six listeners, two per priority-level (in this example, `u32` from 1 to 3).
/// Every listener owns a reference to a record-book and will insert their name upon receiving
/// an event.
/// Hence dispatching mentioned event-type shall result in a vector with our listener's name
/// ordered by their priority-level.
#[test]
fn order_test() {
    let names_record = Arc::new(Mutex::new(Vec::new()));

    let listener_c = Arc::new(Mutex::new(TestListener { name: "1".to_string(), name_record: Arc::clone(&names_record) }));
    let listener_a = Arc::new(Mutex::new(TestListener { name: "3".to_string(), name_record: Arc::clone(&names_record) }));
    let listener_b = Arc::new(Mutex::new(TestListener { name: "2".to_string(), name_record: Arc::clone(&names_record) }));
    let listener_aa = Arc::new(Mutex::new(TestListener { name: "3".to_string(), name_record: Arc::clone(&names_record) }));
    let listener_cc = Arc::new(Mutex::new(TestListener { name: "1".to_string(), name_record: Arc::clone(&names_record) }));
    let listener_bb = Arc::new(Mutex::new(TestListener { name: "2".to_string(), name_record: Arc::clone(&names_record) }));

    let mut dispatcher = PriorityEventDispatcher::<u32, Events>::new();

    {
        dispatcher.add_listener(Events::Event, &listener_a, 3);
        dispatcher.add_listener(Events::Event, &listener_aa, 3);
        dispatcher.add_listener(Events::Event, &listener_b, 2);
        dispatcher.add_listener(Events::Event, &listener_c, 1);
        dispatcher.add_listener(Events::Event, &listener_cc, 1);
        dispatcher.add_listener(Events::Event, &listener_bb, 2);
    }

    dispatcher.dispatch_event(&Events::Event);

    let names_record = names_record.try_lock().unwrap();

    assert_eq!(names_record[0], "3");
    assert_eq!(names_record[1], "3");
    assert_eq!(names_record[2], "2");
    assert_eq!(names_record[3], "2");
    assert_eq!(names_record[4], "1");
    assert_eq!(names_record[5], "1");
}