//! This example shows you how to use the parallel event-dispatcher: `ParallelDispatcher`.
//! We will dispatch enums to trait-objects and closures.
//!
//! The `ParallelDispatcher` will dispatch to all listeners in parallel.
//! For synchronous behaviour look at the `sync_dispatcher`-example, using the `Dispatcher`.
//!
//! I will often use the term `listener`, it describes an event-receiver,
//! the dispatcher dispatches events to such listeners.
//! While most types need to implement the `Listener`-trait,
//! closures can also become a listener.

use hey_listen::{
    sync::{ParallelDispatchResult, ParallelDispatcher, ParallelListener},
    RwLock,
};
use std::sync::{Arc, Weak};

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
// The method only gets an immutable reference to self, if we want to mutate, we will need
// to implement the trait for a Mutex or RwLock wrapping our Listener.
impl ParallelListener<Event> for Arc<RwLock<ListenerStruct>> {
    fn on_event(&self, _event: &Event) -> Option<ParallelDispatchResult> {
        println!("{}", self.read().number);

        self.write().number += 1;

        // At the end, we have to return an `Option<SyncDispatchResult>` request back to
        // the dispatcher.
        // This request gives an instruction back to the dispatcher, here are the variants:
        // - `ParallelDispatchResult::StopListening` to automatically stop listening.
        None
    }
}

impl ParallelListener<Event> for Weak<RwLock<ListenerStruct>> {
    fn on_event(&self, _event: &Event) -> Option<ParallelDispatchResult> {
        if let Some(strong) = self.upgrade() {
            println!("{}", strong.read().number);

            None
        } else {
            Some(ParallelDispatchResult::StopListening)
        }
    }
}

impl ParallelListener<Event>
    for Box<dyn Fn(&Event) -> Option<ParallelDispatchResult> + Send + Sync>
{
    fn on_event(&self, event: &Event) -> Option<ParallelDispatchResult> {
        (&self)(&event)
    }
}

fn main() {
    // Create our dispatcher.
    // The number of threads here is exemplary, one should figure out what's best.
    let mut dispatcher = ParallelDispatcher::<Event>::new(2).expect("Failed to build threadpool");

    // We add listeners, each assigned with a different number.
    let listener_1 = Arc::new(RwLock::new(ListenerStruct { number: 0 }));
    let listener_2 = Arc::new(RwLock::new(ListenerStruct { number: 1 }));

    // Our closure gets its unique number as well.
    let listener_3: Box<dyn Fn(&Event) -> Option<ParallelDispatchResult> + Send + Sync> =
        Box::new(move |_event| {
            println!("3");

            // As we did in the `ParallelListener`-implementation:
            None
        });

    // We add some listeners for our only variant.
    dispatcher.add_listener(Event::EventVariant, Arc::downgrade(&listener_1));
    dispatcher.add_listener(Event::EventVariant, Arc::clone(&listener_2));
    dispatcher.add_listener(Event::EventVariant, listener_3);

    // Let's remember that we gave every listener their own unique number
    // and added in order of their number.
    // If we dispatch now, the numbers can be out of order due to
    // parallel dispatching.
    // It can help to repeat the dispatch to see the effect.
    dispatcher.dispatch_event(&Event::EventVariant);
}
