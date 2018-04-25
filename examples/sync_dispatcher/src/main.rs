//! This example shows you how to use the synchronous event-dispatcher: `EventDispatcher`.
//! We will use enums and trait objects.
//!
//! The `EventDispatcher` itself can only dispatch to one listener at a time,
//! thus referred to as synchronous.
//! For parallel behaviour look at the `parallel_dispatcher`-example, using the
//! `ParallelEventDispatcher`.
//!
//! I will often use the term `listener`, it describes an event-receiver,
//! the dispatcher dispatches events to such listeners.
//! While most types need to implement the `Listener`-trait,
//! closures can also become a listener.

extern crate hey_listen;
// `hey_listen` uses parking_lot's `Mutex` instead of `std::sync::Mutex`.
extern crate parking_lot;

use hey_listen::{EventDispatcher, Listener, SyncDispatcherRequest};
use std::sync::Arc;
use parking_lot::Mutex;

// This is our event-enum, it will represent possible events
// a single event-dispatcher can dispatch.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum EventEnum {
    EventVariantA,
    EventVariantB,
    EventVariantC,
}

// The actual listener.
struct ListenerStruct {}

// This implements the `Listener`-trait, enabling the struct above (`ListenerStruct`)
// to become a trait object when starting listening.
impl Listener<EventEnum> for ListenerStruct {
    fn on_event(&mut self, _event: &EventEnum) -> Option<SyncDispatcherRequest> {
        // Do whatever you want inside here, you can even access the struct's fields.
        // Be aware, the event is immutable.
        println!("I'm listening! :)");

        // At the end, we have to return and `std::Option` request back to
        // the dispatcher.
        // This could be:
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
    // Create your listener.
    let listener = Arc::new(Mutex::new(ListenerStruct {}));

    // Create your dispatcher and define the generic type what the dispatcher
    // shall accept as dispatchable type, it's our declared `EventEnum` in this
    // example.
    let mut dispatcher: EventDispatcher<EventEnum> = EventDispatcher::default();

    // Make your listener start listening.
    dispatcher.add_listener(EventEnum::EventVariantA, &listener);

    // Listeners can listen to multiple variants at once:
    dispatcher.add_listener(EventEnum::EventVariantB, &listener);

    // Dispatches our events to all listeners.
    dispatcher.dispatch_event(&EventEnum::EventVariantA);
    dispatcher.dispatch_event(&EventEnum::EventVariantB);

    // If you want to work with a closure, you can do the following:
    let listening_closure = Box::new(move |event: &EventEnum| {
        // Be aware, since enum's variants are no types,
        // whenever you want to work with the enum,
        // you need to pattern-match it of if-let-bind in order to find its variant,
        // even if you listen to only one variant.
        let event_name = match *event {
            EventEnum::EventVariantA => "A".to_string(), // we won't listen to this ...
            EventEnum::EventVariantB => "B".to_string(), // ... nor to this.
            EventEnum::EventVariantC => "C, as in closure".to_string(),
        };

        println!("Received event-variant: {}!", event_name);

        // As we did in the `Listener`-implementation:
        None
    });

    // Closures require the `add_fn`-method instead `add_listener`.
    dispatcher.add_fn(EventEnum::EventVariantC, listening_closure);

    // Nonetheless, you can trigger them with the same `dispatch_event`-method
    // as used for trait objects.
    dispatcher.dispatch_event(&EventEnum::EventVariantC);
}
