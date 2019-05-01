//! This example shows you how to use the synchronous prioritised event-dispatcher: `PriorityDispatcher`.
//! We will dispatch enums to trait-objects and closures.
//!
//! The `PriorityDispatcher` itself can only dispatch to one listener at a time,
//! thus referred to as synchronous.
//! We will dispatch to lower priority-levels first.
//!
//! I will often use the term `listener`, it describes an event-receiver,
//! the dispatcher dispatches events to such listeners.
//! While most types need to implement the `Listener`-trait,
//! closures can also become a listener.

use hey_listen::{
    sync::{Listener, PriorityDispatcher, SyncDispatcherRequest},
    RwLock,
};
use std::{
    hash::{Hash, Hasher},
    mem::discriminant,
    sync::Arc,
};

// This is our event-enum, it will represent possible events a single
// event-dispatcher can dispatch.
#[derive(Clone, Debug)]
enum EventEnum {
    EventVariant(usize),
}

// We manually implement the `Hash`-, `Eq`-, and `PartialEq` to
// bypass the value `EventVariant` has.
impl Hash for EventEnum {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl PartialEq for EventEnum {
    fn eq(&self, other: &EventEnum) -> bool {
        discriminant(self) == discriminant(other)
    }
}

impl Eq for EventEnum {}

// The actual listener.
struct ListenerStruct {}

// This implements the `Listener`-trait, enabling the struct above (`ListenerStruct`)
// to become a trait-object when starting listening.
impl Listener<EventEnum> for ListenerStruct {
    fn on_event(&mut self, event: &EventEnum) -> Option<SyncDispatcherRequest> {
        // Do whatever you want inside here, you can even access the struct's fields.
        // Be aware, the event is immutable.

        match *event {
            EventEnum::EventVariant(value) => {
                println!("I'm listening and received event with value: {}.", value)
            }
        }

        // At the end, we have to return an `Option<SyncDispatcherRequest>` request back to
        // the dispatcher.
        // This request gives an instruction back to the dispatcher, here are the variants:
        // - `SyncDispatcherRequest::StopListening` to automatically
        // stop listening.
        // - `SyncDispatcherRequest::StopPropagation` stops the dispatcher from dispatching
        // to other listeners in this instance of dispatching.
        // - `SyncDispatcherRequest::StopListeningAndPropagation` combines the first and second.
        // - `None`, equals to sending no request to the dispatcher, everything will stay as it is.
        None
    }
}

fn main() {
    // Create our listener.
    let listener = Arc::new(RwLock::new(ListenerStruct {}));

    // Create our dispatcher, specify that we use `u32` as order-type
    // and `EventEnum` as event-enum.
    let mut dispatcher: PriorityDispatcher<u32, EventEnum> =
        PriorityDispatcher::default();

    // Start listening to a listener and decide their dispatch-priority, here level `1`.
    // The value we give `EventVariant` is not important for adding a listener,
    // since we implemented `Hash`-, `Eq`-, and `PartialEq`-trait in a manner that
    // we bypass fields for comparison but compares variants.
    dispatcher.add_listener(EventEnum::EventVariant(0), &listener, 1);

    // If we want to work with a closure, we can do the following:
    let listening_closure = Box::new(move |event: &EventEnum| {
        // We have to be awar that an enum's variants are no types,
        // whenever we want to work with the enum we need to
        // pattern-match or if-let-bind the enum in order to use its variant.
        let event_name = match *event {
            EventEnum::EventVariant(value) => value.to_string(),
        };

        println!("Received event-variant with value: {}!", event_name);

        // As we did in the `Listener`-implementation:
        None
    });

    // Closures require the `add_fn`-method instead `add_listener`.
    dispatcher.add_fn(EventEnum::EventVariant(0), listening_closure, 3);

    // Dispatches our events to all listeners.
    dispatcher.dispatch_event(&EventEnum::EventVariant(1));
}
