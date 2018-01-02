//! hey_listen is an event dispatcher for types.
//!
//! # Usage
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hey_listen = "0.1"
//! parking_lot = "^0.5"
//! ```
//!
//! and this to your crate's root:
//!
//! ```rust,ignore
//! extern crate hey_listen;
//! extern crate parking_lot;
//! ```
//!
//! # Example
//! Here is a quick example on how to use the event-dispatcher:
//!
//! ```rust
//! extern crate hey_listen;
//! extern crate parking_lot;
//!
//! use hey_listen::Listener;
//! use hey_listen::EventDispatcher;
//! use std::sync::Arc;
//! use parking_lot::Mutex;
//!
//! #[derive(Clone, Eq, Hash, PartialEq)]
//! enum EventEnum {
//!     EventVariant,
//! }
//!
//! struct ListenerStruct {}
//!
//! impl Listener<EventEnum> for ListenerStruct {
//!     fn on_event(&mut self, event: &EventEnum) {
//!         println!("I'm listening! :)");
//!     }
//! }
//!
//! fn main() {
//!     let listener = Arc::new(Mutex::new(ListenerStruct {}));
//!     let mut dispatcher: EventDispatcher<EventEnum> = EventDispatcher::new();
//!
//!     dispatcher.add_listener(EventEnum::EventVariant, &listener);
//! }
//! ```
extern crate parking_lot;

use std::sync::{Weak, Arc};
use std::hash::Hash;
use std::collections::HashMap;
use std::marker::Send;
use parking_lot::Mutex;

type ListenerMap<T> = HashMap<T, Vec<Weak<Mutex<Listener<T>>>>>;

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
pub trait Listener<T>
    where T: PartialEq + Eq + Hash + Clone + Send + 'static {
    /// This function will be dispatched once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&mut self, event: &T);
}

/// Owns a map of all listened event-variants and
/// [`Weak`]-references to their listeners.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
#[derive(Default)]
pub struct EventDispatcher<T>
    where T: PartialEq + Eq + Hash + Clone + Send + 'static {
    events: ListenerMap<T>
}

unsafe impl<T: PartialEq + Eq + Hash + Clone + Send + 'static> Send for EventDispatcher<T> {}

impl<T> EventDispatcher<T>
    where T: PartialEq + Eq + Hash + Clone + Send + 'static {
    pub fn new() -> EventDispatcher<T> {
        EventDispatcher {
            events: ListenerMap::new()
        }
    }

    /// Adds a [`Listener`] to listen for an `event_identifier`.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// **Note**: If your `Enum` owns fields you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore fields,
    /// see second example for an implementation-suggestion.
    ///
    /// # Examples
    ///
    /// Adding a [`Listener`] to the dispatcher:
    ///
    /// ```rust
    /// extern crate hey_listen;
    /// extern crate parking_lot;
    /// use std::sync::Arc;
    ///
    /// use hey_listen::Listener;
    /// use hey_listen::EventDispatcher;
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum EventEnum {
    ///     EventVariant,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// impl Listener<EventEnum> for ListenerStruct {
    ///     fn on_event(&mut self, event: &EventEnum) {}
    /// }
    ///
    /// fn main() {
    ///     let listener = Arc::new(parking_lot::Mutex::new(ListenerStruct {}));
    ///     let mut dispatcher: EventDispatcher<EventEnum> = EventDispatcher::new();
    ///
    ///     dispatcher.add_listener(EventEnum::EventVariant, &listener);
    /// }
    /// ```
    ///
    /// Declaring your own [`Hash`]- and [`PartialEq`]-trait to bypass
    /// hashing on fields:
    ///
    /// ```rust
    /// use std::hash::{Hash, Hasher};
    /// use std::mem::discriminant;
    ///
    /// #[derive(Clone)]
    /// enum Test {
    ///     TestVariant(i32),
    /// }
    ///
    /// impl Hash for Test {
    ///     fn hash<H: Hasher>(&self, _state: &mut H) {}
    /// }
    ///
    /// impl PartialEq for Test {
    ///     fn eq(&self, other: &Test) -> bool {
    ///         discriminant(self) == discriminant(other)
    ///     }
    /// }
    ///
    /// impl Eq for Test {}
    /// ```
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_listener<D: Listener<T> + 'static>(&mut self, event_identifier: T, listener: &Arc<Mutex<D>>) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.push(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)));

            return;
        }

        self.events.insert(event_identifier, vec!(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>))));
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(listener_collection) = self.events.get_mut(event_identifier) {
            let mut found_invalid_weak_ref = false;

            for listener in listener_collection.iter()  {

                if let Some(listener_rc) = listener.upgrade() {
                    let mut listener = listener_rc.lock();
                    listener.on_event(event_identifier);
                } else {
                    found_invalid_weak_ref = true;
                }
            }

            if found_invalid_weak_ref {
                listener_collection.retain(|listener| {
                    Weak::clone(listener).upgrade().is_some()
                });
            }
        }
    }
}
