
use hey_listen::{
    sync::{
        AsyncDispatcher as Dispatcher,
        AsyncListener as Listener,
        AsyncDispatchResult as DispatcherRequest
    },
    RwLock,
};
use std::sync::Arc;
use tokio::prelude::*;

// `async trait`s are not supported on Rust `1.39.0`.
// We will use this macro to bypass this.
// It requires us to prepend an `impl`-block with `#[async_trait]` in order
// to write `async fn`.
use hey_listen::async_trait;

// This is our event-enum, it will represent possible events
// a single event-dispatcher can dispatch.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum EventEnum {
    EventVariantA,
    EventVariantB,
    EventVariantC,
}

// The actual listener.
struct BlockingListener {}

// This implements the `Listener`-trait, enabling the struct above (`ListenerStruct`)
// to become a trait-object when starting listening.
#[async_trait]
impl Listener<EventEnum> for BlockingListener {
    async fn on_event(&mut self, _event: &EventEnum) -> Option<DispatcherRequest> {
        tokio::time::delay_for(std::time::Duration::from_secs(10)).await;

        None
    }
}

// The actual listener.
struct NormalListener {}

// This implements the `Listener`-trait, enabling the struct above (`ListenerStruct`)
// to become a trait-object when starting listening.
#[async_trait]
impl Listener<EventEnum> for NormalListener {
    async fn on_event(&mut self, _event: &EventEnum) -> Option<DispatcherRequest> {
        println!("I'm listening!");

        None
    }
}

#[tokio::main]
async fn main() {
    // Create your listener.
    let blocking_listener = Arc::new(RwLock::new(BlockingListener {}));
    let normal_listener = Arc::new(RwLock::new(NormalListener {}));

    // Create your dispatcher and define the generic type what the dispatcher
    // shall accept as dispatchable type, it's our declared `EventEnum` in this
    // example.
    let mut dispatcher: Dispatcher<EventEnum> = Dispatcher::default();

    // Make your listener start listening.
    dispatcher.add_listener(EventEnum::EventVariantA, &blocking_listener);

    // Listeners can listen to multiple variants at once:
    dispatcher.add_listener(EventEnum::EventVariantA, &normal_listener);

    // Dispatches our events to all listeners.
    dispatcher.dispatch_event(&EventEnum::EventVariantA).await;
}
