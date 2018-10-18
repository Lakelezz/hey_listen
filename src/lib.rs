//! `Hey_listen` is a collection of event-dispatchers aiming to suit all needs!
//!
//! Covering synchronous, parallel, and sync-prioritised dispatching to
//! Closures, Enums, Structs, and every other type supporting `trait`-implementation.
//!
//! View the [`examples`] on how to use each dispatcher.
//!
//! # Usage
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hey_listen = "0.2.0"
//! ```
//!
//! and this to your crate's root:
//!
//! ```rust,ignore
//! extern crate hey_listen;
//! ```
//!
//! # Example
//! Here is a quick example on how to use the sync event-dispatcher:
//!
//! ```rust
//! extern crate hey_listen;
//!
//! use hey_listen::{Listener, EventDispatcher, Mutex, SyncDispatcherRequest};
//! use std::sync::Arc;
//!
//! #[derive(Clone, Eq, Hash, PartialEq)]
//! enum Event {
//!     EventType,
//! }
//!
//! struct ListenerStruct {}
//!
//! impl Listener<Event> for ListenerStruct {
//!     fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> {
//!         println!("I'm listening! :)");
//!
//!         None
//!     }
//! }
//!
//! fn main() {
//!     let listener = Arc::new(Mutex::new(ListenerStruct {}));
//!     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
//!
//!     dispatcher.add_listener(Event::EventType, &listener);
//! }
//!
//! ```
//! [`examples`]: https://github.com/Lakelezz/hey_listen/tree/master/examples
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate parking_lot;
extern crate rayon;

mod parallel;
mod sync;

pub use self::{
    parallel::{
        dispatcher::ParallelEventDispatcher, ParallelDispatcherRequest, ParallelListener,
    },
    sync::{
        dispatcher::EventDispatcher, priority_dispatcher::PriorityEventDispatcher, Listener,
        SyncDispatcherRequest,
    },
};
pub use parking_lot::Mutex;
