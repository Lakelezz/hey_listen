//! `Hey_listen` is a collection of event-dispatchers suiting all needs: sync, parallel and prioritised.
//! Dispatching to Closures, Enums, Structs and every other type supporting `trait`-implementation.
//!
//! # Usage
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hey_listen = "0.2.0"
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
//! Here is a quick example on how to use the sync event-dispatcher:
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
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate parking_lot;
extern crate rayon;

use std::error::Error;
use std::sync::{Arc, Weak};
use std::hash::Hash;
use std::collections::{BTreeMap, HashMap};
use parking_lot::Mutex;
use rayon::{join, ThreadPool};
use rayon::prelude::*;

type ListenerMap<T> = HashMap<T, FnsAndTraits<T>>;
type PriorityListenerMap<P, T> = HashMap<T, BTreeMap<P, FnsAndTraits<T>>>;
type EventFunction<T> = Vec<Box<Fn(&T) -> Result<(), SyncDispatcherRequest>>>;
type ParallelListenerMap<T> = HashMap<T, ParallelFnsAndTraits<T>>;
type ParallelEventFunction<T> = Vec<Arc<Fn(&T) -> Option<ParallelDispatcherRequest> + Send + Sync>>;

/// An `enum` returning a request from a [`Listener`] to its `sync` event-dispatcher.
///
/// `StopListening` will remove your [`Listener`] from the
/// event-dispatcher.
///
/// `StopPropagation` will stop dispatching of the just now
/// acquired `Event`.
///
/// `StopListeningAndPropagation` a combination of
/// `StopListening` and `StopPropagation`.
///
/// [`Listener`]: trait.Listener.html
pub enum SyncDispatcherRequest {
    StopListening,
    StopPropagation,
    StopListeningAndPropagation,
}

/// An `enum` returning a request from a [`Listener`] to its `async` event-dispatcher.
///
/// `StopListening` will remove your [`Listener`] from the
/// event-dispatcher.
///
/// **Note**:
/// Opposed to `SyncDispatcherRequest` a [`Listener`] cannot
/// stop propagation as the propagation is happening parallel.
///
/// [`Listener`]: trait.Listener.html
pub enum ParallelDispatcherRequest {
    StopListening,
}

/// Yields closures and trait-objects.
struct FnsAndTraits<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    traits: Vec<Weak<Mutex<Listener<T>>>>,
    fns: EventFunction<T>,
}

impl<T> FnsAndTraits<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    fn new_with_traits(trait_objects: Vec<Weak<Mutex<Listener<T>>>>) -> Self {
        FnsAndTraits {
            traits: trait_objects,
            fns: vec![],
        }
    }

    fn new_with_fns(fns: EventFunction<T>) -> Self {
        FnsAndTraits {
            traits: vec![],
            fns,
        }
    }
}

/// Yields `Send` and `Sync` closures and trait-objects.
pub struct ParallelFnsAndTraits<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    traits: Vec<Weak<Mutex<ParallelListener<T> + Send + Sync + 'static>>>,
    fns: ParallelEventFunction<T>,
}

impl<T> ParallelFnsAndTraits<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    fn new_with_traits(
        trait_objects: Vec<Weak<Mutex<ParallelListener<T> + Send + Sync + 'static>>>,
    ) -> Self {
        ParallelFnsAndTraits {
            traits: trait_objects,
            fns: vec![],
        }
    }

    fn new_with_fns(fns: ParallelEventFunction<T>) -> Self {
        ParallelFnsAndTraits {
            traits: vec![],
            fns,
        }
    }
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
pub trait Listener<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&mut self, event: &T);
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
pub trait ParallelListener<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&mut self, event: &T) -> Option<ParallelDispatcherRequest>;
}

/// Owns a map of all listened event-variants,
/// [`Weak`]-references to their listeners and [`Fn`]s.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
pub struct EventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: ListenerMap<T>,
}

impl<T> EventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    pub fn new() -> EventDispatcher<T> {
        EventDispatcher {
            events: ListenerMap::new(),
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
    pub fn add_listener<D: Listener<T> + Sync + Sync + 'static>(
        &mut self,
        event_identifier: T,
        listener: &Arc<Mutex<D>>,
    ) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.traits.push(Arc::downgrade(
                &(Arc::clone(listener) as Arc<Mutex<Listener<T>>>),
            ));

            return;
        }

        self.events.insert(
            event_identifier,
            FnsAndTraits::new_with_traits(vec![
                Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)),
            ]),
        );
    }

    /// Adds a [`Fn`] to listen for an `event_identifier`.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// **Note**: If your `Enum` owns fields you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore fields.
    ///
    /// # Examples
    ///
    /// Adding a [`Fn`] to the dispatcher:
    ///
    /// ```rust
    /// extern crate hey_listen;
    /// extern crate parking_lot;
    ///
    /// use hey_listen::EventDispatcher;
    /// use hey_listen::SyncDispatcherRequest;
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
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_fn(
        &mut self,
        event_identifier: T,
        function: Box<Fn(&T) -> Result<(), SyncDispatcherRequest>>,
    ) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.fns.push(function);

            return;
        }

        self.events
            .insert(event_identifier, FnsAndTraits::new_with_fns(vec![function]));
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`Fn`]s returning [`Result`] with `Ok(())` will be retained
    /// and `Err(Error::StoppedListening)` will cause them to
    /// be removed from the event-dispatcher.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    /// [`Error`]: enum.Error.html
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(listener_collection) = self.events.get_mut(event_identifier) {
            let mut found_invalid_weak_ref = false;

            for listener in (&listener_collection.traits).iter() {
                if let Some(listener_arc) = listener.upgrade() {
                    let mut listener = listener_arc.lock();
                    listener.on_event(event_identifier);
                } else {
                    found_invalid_weak_ref = true;
                }
            }

            listener_collection
                .fns
                .retain(|callback| callback(event_identifier).is_ok());

            if found_invalid_weak_ref {
                listener_collection
                    .traits
                    .retain(|listener| Weak::clone(listener).upgrade().is_some());
            }
        }
    }
}

/// Owns a map of all listened event-variants,
/// [`Weak`]-references to their listeners and [`Fn`]s.
/// Opposed to [`EventListener`], this structure utilises one [`BTreeMap`] per
/// event-type to order listeners by a given priority-level.
///
/// **Note**: Consider implementing your own [`Ord`]-trait, if you
/// want a different kind of order.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
/// [`Ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
/// [`EventListener`]: struct.EventDispatcher.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
#[derive(Default)]
pub struct PriorityEventDispatcher<P, T>
where
    P: Ord,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: PriorityListenerMap<P, T>,
}

unsafe impl<P: Ord, T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static> Send
    for PriorityEventDispatcher<P, T> {
}

impl<P, T> PriorityEventDispatcher<P, T>
where
    P: Ord + Clone,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    pub fn new() -> PriorityEventDispatcher<P, T> {
        PriorityEventDispatcher {
            events: PriorityListenerMap::new(),
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
    pub fn add_listener<D: Listener<T> + Send + Sync + 'static>(
        &mut self,
        event_identifier: T,
        listener: &Arc<Mutex<D>>,
        priority: P,
    ) {
        if let Some(prioritised_listener_collection) = self.events.get_mut(&event_identifier) {
            if let Some(priority_level_collection) =
                prioritised_listener_collection.get_mut(&priority)
            {
                priority_level_collection.traits.push(Arc::downgrade(
                    &(Arc::clone(listener) as Arc<Mutex<Listener<T>>>),
                ));

                return;
            }
            prioritised_listener_collection.insert(
                priority.clone(),
                FnsAndTraits::new_with_traits(vec![
                    Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)),
                ]),
            );
            return;
        }

        let mut b_tree_map = BTreeMap::new();
        b_tree_map.insert(
            priority,
            FnsAndTraits::new_with_traits(vec![
                Arc::downgrade(&(Arc::clone(listener) as Arc<Mutex<Listener<T>>>)),
            ]),
        );
        self.events.insert(event_identifier, b_tree_map);
    }

    /// Adds a [`Fn`] to listen for an `event_identifier`, considering
    /// a given `priority` implementing the [`Ord`]-trait in order to sort dispatch-order.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// # Examples
    ///
    /// Adding an [`Fn`] to the dispatcher:
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
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_fn(
        &mut self,
        event_identifier: T,
        function: Box<Fn(&T) -> Result<(), SyncDispatcherRequest>>,
        priority: P,
    ) {
        if let Some(prioritised_listener_collection) = self.events.get_mut(&event_identifier) {
            if let Some(priority_level_collection) =
                prioritised_listener_collection.get_mut(&priority)
            {
                priority_level_collection.fns.push(function);

                return;
            }
            prioritised_listener_collection
                .insert(priority.clone(), FnsAndTraits::new_with_fns(vec![function]));
            return;
        }

        let mut b_tree_map = BTreeMap::new();
        b_tree_map.insert(priority, FnsAndTraits::new_with_fns(vec![function]));
        self.events.insert(event_identifier, b_tree_map);
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`Fn`]s returning [`Result`] with `Ok(())` will be retained
    /// and `Err(Error::StoppedListening)` will cause them to
    /// be removed from the event-dispatcher.
    ///
    /// **Notice**: [`Listener`]s will called ordered by their priority-level.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(prioritised_listener_collection) = self.events.get_mut(event_identifier) {
            for (_, listener_collection) in prioritised_listener_collection.iter_mut() {
                let mut found_invalid_weak_ref = false;

                for listener in (&listener_collection.traits).iter() {
                    if let Some(listener_arc) = listener.upgrade() {
                        let mut listener = listener_arc.lock();
                        listener.on_event(event_identifier);
                    } else {
                        found_invalid_weak_ref = true;
                    }
                }

                if found_invalid_weak_ref {
                    listener_collection
                        .traits
                        .retain(|listener| Weak::clone(listener).upgrade().is_some());
                }
            }
        }
    }
}

/// Owns a map of all listened event-variants,
/// [`Weak`]-references to their listeners and [`Fn`]s.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
#[derive(Default)]
pub struct ParallelEventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: ParallelListenerMap<T>,
}

impl<T> ParallelEventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    pub fn new() -> ParallelEventDispatcher<T> {
        ParallelEventDispatcher {
            events: ParallelListenerMap::new(),
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
    pub fn add_listener<D: ParallelListener<T> + Send + Sync + 'static>(
        &mut self,
        event_identifier: T,
        listener: &Arc<Mutex<D>>,
    ) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.traits.push(Arc::downgrade(
                &(Arc::clone(listener) as Arc<Mutex<ParallelListener<T> + Send + Sync + 'static>>),
            ));

            return;
        }

        self.events.insert(
            event_identifier,
            ParallelFnsAndTraits::new_with_traits(vec![
                Arc::downgrade(
                    &(Arc::clone(listener)
                        as Arc<Mutex<ParallelListener<T> + Send + Sync + 'static>>),
                ),
            ]),
        );
    }

    /// Adds a [`Fn`] to listen for an `event_identifier`.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// **Note**: If your `Enum` owns fields you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore fields.
    ///
    /// # Examples
    ///
    /// Adding a [`Fn`] to the dispatcher:
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
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    pub fn add_fn(
        &mut self,
        event_identifier: T,
        function: Arc<Fn(&T) -> Option<ParallelDispatcherRequest> + Send + Sync>,
    ) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.fns.push(function);

            return;
        }

        self.events.insert(
            event_identifier,
            ParallelFnsAndTraits::new_with_fns(vec![function]),
        );
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`Fn`]s returning [`Result`] with `Ok(())` will be retained
    /// and `Err(Error::StoppedListening)` will cause them to
    /// be removed from the event-dispatcher.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    /// [`Error`]: enum.Error.html
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
    /// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(listener_collection) = self.events.get_mut(event_identifier) {
            let mut fns_to_remove = Mutex::new(Vec::new());
            let mut traits_to_remove = Mutex::new(Vec::new());

            join(
                || {
                    listener_collection.traits.par_iter().enumerate().for_each(
                        |(index, listener)| {
                            if let Some(listener_arc) = listener.upgrade() {
                                let mut listener = listener_arc.lock();

                                if let Some(instruction) = listener.on_event(event_identifier) {
                                    match instruction {
                                        ParallelDispatcherRequest::StopListening => {
                                            traits_to_remove.lock().push(index)
                                        }
                                    }
                                }
                            } else {
                                traits_to_remove.lock().push(index)
                            }
                        },
                    )
                },
                || {
                    listener_collection
                        .fns
                        .par_iter()
                        .enumerate()
                        .for_each(|(index, callback)| {
                            if let Some(instruction) = callback(event_identifier) {
                                match instruction {
                                    ParallelDispatcherRequest::StopListening => {
                                        fns_to_remove.lock().push(index);
                                    }
                                }
                            } else {
                                ()
                            }
                        });
                },
            );

            fns_to_remove.lock().iter().for_each(|index| {
                listener_collection.fns.swap_remove(*index);
            });

            traits_to_remove.lock().iter().for_each(|index| {
                listener_collection.traits.swap_remove(*index);
            });
        }
    }
}
