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
//! hey_listen = "0.4.0"
//! ```
//!
//! # Example
//! Here is a quick example on how to use the sync event-dispatcher:
//!
//! ```rust
//! use hey_listen::{
//!    RwLock,
//!    sync::{Listener, Dispatcher, SyncDispatcherRequest},
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
//! let listener = Arc::new(RwLock::new(ListenerStruct {}));
//! let mut dispatcher: Dispatcher<Event> = Dispatcher::default();
//!
//! dispatcher.add_listener(Event::EventType, &listener);
//!
//! ```
//! [`examples`]: https://github.com/Lakelezz/hey_listen/tree/master/examples
#![deny(rust_2018_idioms)]

pub mod rc;
pub mod sync;

pub use parking_lot::{Mutex, RwLock};

// #[cfg(feature = "parallel")]
use rayon::ThreadPoolBuildError;

/// `hey_listen`'s Error collection.
#[derive(Debug)]
pub enum Error {
    /// Error when building a threadpool fails.
    ThreadPoolBuilder(String),
}

// #[cfg(feature = "parallel")]
impl From<ThreadPoolBuildError> for Error {
    fn from(error: ThreadPoolBuildError) -> Self {
        Self::ThreadPoolBuilder(error.to_string())
    }
}
