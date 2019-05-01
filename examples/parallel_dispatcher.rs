//! This example shows you how to use the parallel event-dispatcher: `ParallelEventDispatcher`.
//! We will dispatch enums to trait-objects and closures.
//!
//! The `ParallelEventDispatcher` will dispatch to all listeners in parallel.
//! For synchronous behaviour look at the `sync_dispatcher`-example, using the `EventDispatcher`.
//!
//! I will often use the term `listener`, it describes an event-receiver,
//! the dispatcher dispatches events to such listeners.
//! While most types need to implement the `Listener`-trait,
//! closures can also become a listener.



use hey_listen::{RwLock, ParallelDispatcherRequest, ParallelEventDispatcher, ParallelListener};
use std::sync::Arc;

// This is our event-enum, it will represent possible events a single
// event-dispatcher can dispatch.
#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    EventVariant,
}

// The actual listener one `usize`-field to display
// later when dispatching.
#[derive(Default)]
struct ListenerStruct {
    number: usize,
}

// This implements the `ParallelListener`-trait, enabling the struct above (`ListenerStruct`)
// to become a trait-object when starting listening.
impl ParallelListener<Event> for ListenerStruct {
    fn on_event(&mut self, _event: &Event) -> Option<ParallelDispatcherRequest> {
        println!("{}", self.number);

        // At the end, we have to return an `Option<SyncDispatcherRequest>` request back to
        // the dispatcher.
        // This request gives an instruction back to the dispatcher, here are the variants:
        // - `ParallelDispatcherRequest::StopListening` to automatically stop listening.
        None
    }
}

fn main() {
    // Create our dispatcher.
    let mut dispatcher = ParallelEventDispatcher::<Event>::default();

    // We add listeners, each assigned with a different number.
    let listener_a = Arc::new(RwLock::new(ListenerStruct { number: 0 }));
    let listener_b = Arc::new(RwLock::new(ListenerStruct { number: 1 }));
    let listener_c = Arc::new(RwLock::new(ListenerStruct { number: 2 }));

    // Our closure gets its unique own number as well.
    let closure_a = Box::new(move |_event: &Event| {
        println!("3");

        // As we did in the `ParallelListener`-implementation:
        None
    });

    // We add some listeners for our only variant.
    dispatcher.add_listener(Event::EventVariant, &listener_a);
    dispatcher.add_listener(Event::EventVariant, &listener_b);
    dispatcher.add_listener(Event::EventVariant, &listener_c);

    // Closures require the `add_fn`-method.
    dispatcher.add_fn(Event::EventVariant, closure_a);

    // Let's remember that we gave every listener their own number
    // and added in order of their number.
    // If we dispatch now, the numbers can be out of order due to
    // parallel dispatching.
    // It can help to repeat the dispatch to see the effect.
    dispatcher.dispatch_event(&Event::EventVariant);
}
