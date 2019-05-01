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
//! # Example
//! Here is a quick example on how to use the sync event-dispatcher:
//!
//! ```rust
//! use hey_listen::{
//!    RwLock,
//!    sync::{Listener, EventDispatcher, SyncDispatcherRequest},
//! };
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
//!     let listener = Arc::new(RwLock::new(ListenerStruct {}));
//!     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
//!
//!     dispatcher.add_listener(Event::EventType, &listener);
//! }
//!
//! ```
//! [`examples`]: https://github.com/Lakelezz/hey_listen/tree/master/examples
#![deny(rust_2018_idioms)]

use failure;

pub mod rc;
pub mod sync;

pub use parking_lot::RwLock;
