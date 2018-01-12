//! Hey_listen is an event-dispatcher for Closures, Structs, Enums and other types.
//! On another note, hey_listen supports prioritised/ordered dispatching and aims to add
//! parallel dispatching as well!
//!
//! # Usage
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hey_listen = "0.1.2"
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
//! enum Event {
//!     EventType,
//! }
//!
//! struct ListenerStruct {}
//!
//! impl Listener<Event> for ListenerStruct {
//!     fn on_event(&mut self, event: &Event) {
//!         println!("I'm listening! :)");
//!     }
//! }
//!
//! fn main() {
//!     let listener = Arc::new(Mutex::new(ListenerStruct {}));
//!     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::new();
//!
//!     dispatcher.add_listener(Event::EventType, &listener);
//! }
//!
//! ```
//! Check out Hey_Listen's documentation for further instructions on e.g. prioritised dispatching or
//! using closures.
extern crate parking_lot;

use std::sync::{Weak, Arc};
use std::hash::Hash;
use std::collections::{HashMap, BTreeMap};
use std::marker::Send;
use parking_lot::Mutex;

type ListenerMap<T> = HashMap<T, FnsAndTraits<T>>;
type PriorityListenerMap<P, T> = HashMap<T, BTreeMap<P, FnsAndTraits<T>>>;

pub enum Error {
    StoppedListening,
}

/// Yields closures and trait-objects.
struct FnsAndTraits<T>
    where T: PartialEq + Eq + Hash + Clone + Send + 'static {
    traits: Vec<Weak<Mutex<Listener<T>>>>,
    fns: Vec<Box<Fn(&T) -> Result<(), Error>>>,
}

impl<T> FnsAndTraits<T>
    where T: PartialEq + Eq + Hash + Clone + Send + 'static {
    fn new_with_traits(trait_objects: Vec<Weak<Mutex<Listener<T>>>>) -> Self {
        FnsAndTraits {
            traits: trait_objects,
            fns: vec!(),
        }
    }

    fn new_with_fns(fns: Vec<Box<Fn(&T) -> Result<(), Error>>>) -> Self {
        FnsAndTraits {
            traits: vec!(),
            fns: fns,
        }
    }
}

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
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// impl Listener<Event> for ListenerStruct {
    ///     fn on_event(&mut self, event: &Event) {}
    /// }
    ///
    /// fn main() {
    ///     let listener = Arc::new(parking_lot::Mutex::new(ListenerStruct {}));
    ///     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::new();
    ///
    ///     dispatcher.add_listener(Event::EventType, &listener);
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
    /// enum Event {
    ///     TestVariant(i32),
    /// }
    ///
    /// impl Hash for Event {
    ///     fn hash<H: Hasher>(&self, _state: &mut H) {}
    /// }
    ///
    /// impl PartialEq for Event {
    ///     fn eq(&self, other: &Event) -> bool {
    ///         discriminant(self) == discriminant(other)
    ///     }
    /// }
    ///
    /// impl Eq for Event {}
    /// ```
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_listener<D: Listener<T> + 'static>(&mut self, event_identifier: T, listener: &Arc<Mutex<D>>) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.traits.push(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)));

            return;
        }

        self.events.insert(event_identifier, FnsAndTraits::new_with_traits((vec!(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>))))));
    }

    /// Adds a [`fn`] to listen for an `event_identifier`.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// **Note**: If your `Enum` owns fields you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore fields.
    ///
    /// # Examples
    ///
    /// Adding a [`fn`] to the dispatcher:
    ///
    /// ```rust
    /// extern crate hey_listen;
    /// extern crate parking_lot;
    ///
    /// use hey_listen::EventDispatcher;
    /// use hey_listen::Error;
    /// use std::sync::Arc;
    /// use parking_lot::Mutex;
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct EventListener {
    ///     used_method: bool,
    /// }
    ///
    /// impl EventListener {
    ///     fn test_method(&mut self, _event: &Event) {
    ///         self.used_method = true;
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let listener = Arc::new(Mutex::new(EventListener { used_method: false }));
    ///     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::new();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Box::new(move |event: &Event| -> Result<(), Error> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.lock().test_method(&event);
    ///             Ok(())
    ///         } else {
    ///             Err(Error::StoppedListening)
    ///         }
    ///     });
    ///
    ///     dispatcher.add_fn(Event::EventType, closure);
    /// }
    /// ```
    ///
    /// [`fn`]: https://doc.rust-lang.org/std/primitive.fn.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_fn(&mut self, event_identifier: T, function: Box<Fn(&T) -> Result<(), Error>>) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.fns.push(function);

            return;
        }

        self.events.insert(event_identifier, FnsAndTraits::new_with_fns(vec!(function)));
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method
    /// and all [`fn`]s in a [`Box`] that return a result with
    /// [`Error`] as `Err`.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    /// [`fn`]: https://doc.rust-lang.org/std/primitive.fn.html
    /// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
    /// [`Error`]: enum.Error.html
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(listener_collection) = self.events.get_mut(event_identifier) {
            let mut found_invalid_weak_ref = false;

            for listener in listener_collection.traits.iter() {

                if let Some(listener_arc) = listener.upgrade() {
                    let mut listener = listener_arc.lock();
                    listener.on_event(event_identifier);
                } else {
                    found_invalid_weak_ref = true;
                }
            }

            listener_collection.fns.retain(|callback| {
                callback(event_identifier).is_ok()
            });

            if found_invalid_weak_ref {
                listener_collection.traits.retain(|listener| {
                    Weak::clone(listener).upgrade().is_some()
                });
            }
        }
    }
}

/// Owns a map of all listened event-variants and
/// [`Weak`]-references to their listeners but opposed to
/// [`EventListener`], this structure utilises one [`BTreeMap`] per
/// event-type to order listeners by a given priority-level.
///
/// **Note**: Consider implementing your own [`Ord`]-trait, if you
/// want a different kind of order.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
/// [`Ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
/// [`EventListener`]: struct.EventDispatcher.html
#[derive(Default)]
pub struct PriorityEventDispatcher<P, T>
    where P: Ord,
        T: PartialEq + Eq + Hash + Clone + Send + 'static {
    events: PriorityListenerMap<P, T>
}

unsafe impl<P: Ord, T: PartialEq + Eq + Hash + Clone + Send + 'static> Send for PriorityEventDispatcher<P, T> {}

impl<P, T> PriorityEventDispatcher<P, T>
    where P: Ord + Clone,
        T: PartialEq + Eq + Hash + Clone + Send + 'static {
    pub fn new() -> PriorityEventDispatcher<P, T> {
        PriorityEventDispatcher {
            events: PriorityListenerMap::new()
        }
    }

    /// Adds a [`Listener`] to listen for an `event_identifier`, considering
    /// a given `priority` implementing the [`Ord`]-trait, to sort dispatch-order.
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
    /// use hey_listen::PriorityEventDispatcher;
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// impl Listener<Event> for ListenerStruct {
    ///     fn on_event(&mut self, event: &Event) {}
    /// }
    ///
    /// fn main() {
    ///     let listener = Arc::new(parking_lot::Mutex::new(ListenerStruct {}));
    ///     let mut dispatcher: PriorityEventDispatcher<u32, Event> = PriorityEventDispatcher::new();
    ///
    ///     dispatcher.add_listener(Event::EventType, &listener, 1);
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
    /// enum Event {
    ///     TestVariant(i32),
    /// }
    ///
    /// impl Hash for Event {
    ///     fn hash<H: Hasher>(&self, _state: &mut H) {}
    /// }
    ///
    /// impl PartialEq for Event {
    ///     fn eq(&self, other: &Event) -> bool {
    ///         discriminant(self) == discriminant(other)
    ///     }
    /// }
    ///
    /// impl Eq for Event {}
    /// ```
    ///
    /// [`EventDispatcher`]: struct.EventDispatcher.html
    /// [`Listener`]: trait.Listener.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    /// [`Ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
    pub fn add_listener<D: Listener<T> + 'static>(&mut self, event_identifier: T, listener: &Arc<Mutex<D>>, priority: P) {
        if let Some(prioritised_listener_collection) = self.events.get_mut(&event_identifier) {

            if let Some(priority_level_collection) = prioritised_listener_collection.get_mut(&priority) {
                priority_level_collection.traits.push(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)));

                return;
            }
            prioritised_listener_collection.insert(priority.clone(), FnsAndTraits::new_with_traits(vec!(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)))));
            return;
        }

        let mut b_tree_map = BTreeMap::new();
        b_tree_map.insert(priority, FnsAndTraits::new_with_traits(vec!(Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)))));
        self.events.insert(event_identifier, b_tree_map);
    }

    /// Adds a [`fn`] to listen for an `event_identifier`, considering
    /// a given `priority` implementing the [`Ord`]-trait, to sort dispatch-order.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// # Examples
    ///
    /// Adding a [`fn`] to the dispatcher:
    ///
    /// ```rust
    /// extern crate hey_listen;
    /// extern crate parking_lot;
    ///
    /// use hey_listen::PriorityEventDispatcher;
    /// use hey_listen::Error;
    /// use std::sync::Arc;
    /// use parking_lot::Mutex;
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct EventListener {
    ///     used_method: bool,
    /// }
    ///
    /// impl EventListener {
    ///     fn test_method(&mut self, _event: &Event) {
    ///         self.used_method = true;
    ///     }
    /// }
    ///
    /// fn main() {
    ///     let listener = Arc::new(Mutex::new(EventListener { used_method: false }));
    ///     let mut dispatcher: PriorityEventDispatcher<u32, Event> = PriorityEventDispatcher::new();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Box::new(move |event: &Event| -> Result<(), Error> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.lock().test_method(&event);
    ///             Ok(())
    ///         } else {
    ///             Err(Error::StoppedListening)
    ///         }
    ///     });
    ///
    ///     dispatcher.add_fn(Event::EventType, closure, 1);
    /// }
    /// ```
    ///
    /// [`fn`]: https://doc.rust-lang.org/std/primitive.fn.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_fn(&mut self, event_identifier: T, function: Box<Fn(&T) -> Result<(), Error>>, priority: P) {
        if let Some(prioritised_listener_collection) = self.events.get_mut(&event_identifier) {

            if let Some(priority_level_collection) = prioritised_listener_collection.get_mut(&priority) {
                priority_level_collection.fns.push(function);

                return;
            }
            prioritised_listener_collection.insert(priority.clone(), FnsAndTraits::new_with_fns(vec!(function)));
            return;
        }

        let mut b_tree_map = BTreeMap::new();
        b_tree_map.insert(priority, FnsAndTraits::new_with_fns(vec!(function)));
        self.events.insert(event_identifier, b_tree_map);
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    ///
    /// **Notice**: [`Listener`]s will called ordered by their priority-level.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(prioritised_listener_collection) = self.events.get_mut(event_identifier) {

            for (_, listener_collection) in prioritised_listener_collection.iter_mut() {
                let mut found_invalid_weak_ref = false;

                for listener in listener_collection.traits.iter() {

                    if let Some(listener_arc) = listener.upgrade() {
                        let mut listener = listener_arc.lock();
                        listener.on_event(event_identifier);
                    } else {
                        found_invalid_weak_ref = true;
                    }
                }

                if found_invalid_weak_ref {
                    listener_collection.traits.retain(|listener| {
                        Weak::clone(listener).upgrade().is_some()
                    });
                }
            }
        }
    }
}