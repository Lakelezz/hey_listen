//! `Hey_listen` is a collection of event-dispatchers aiming to suit all needs!
//!
//! Covering synchronous, parallel, and sync-prioritised dispatching to
//! Closures, Enums, Structs, and every other type supporting `trait`-implementation.
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
//!     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
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

/// Iterates over the passed `vec` and applies `function` to each element.
/// `function`'s returned [`SyncDispatcherRequest`] will instruct
/// a procedure depending on its variant:
///
/// `StopListening`: Removes item from `vec`.
/// `StopPropagation`: Stops further dispatching to other elements
/// in `vec`.
/// `StopListeningAndPropagation`: Execute `StopListening`,
/// then execute `StopPropagation`.
///
/// **Note**: When `StopListening` is being executed,
/// removal of items from `vec` will result use a swap of elements,
/// resulting in an alteration of the order items were originally
/// inserted into `vec`.
///
/// **Note**: Unlike [`retain`], [`execute_sync_dispatcher_requests`]
/// can break the current iteration and is able to match [`SyncDispatcherRequest`]
/// and perform actions based on variants.
///
/// [`retain`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html#method.retain
/// [`SyncDispatcherRequest`]: enum.SyncDispatcherRequest.html
pub(crate) fn execute_sync_dispatcher_requests<T, F>(vec: &mut Vec<T>, mut function: F)
where
    F: FnMut(&T) -> Option<SyncDispatcherRequest>,
{
    let mut index = 0;

    loop {
        if index < vec.len() {
            match function(&vec[index]) {
                None => index += 1,
                Some(SyncDispatcherRequest::StopListening) => {
                    vec.swap_remove(index);
                }
                Some(SyncDispatcherRequest::StopPropagation) => break,
                Some(SyncDispatcherRequest::StopListeningAndPropagation) => {
                    vec.swap_remove(index);
                    break;
                }
            }
        } else {
            break;
        }
    }
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
    fn on_event(&mut self, event: &T) -> Option<SyncDispatcherRequest>;
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

impl<T> Default for EventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static {
    fn default() -> EventDispatcher<T> {
        EventDispatcher {
            events: ListenerMap::new(),
        }
    }
}

impl<T> EventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
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
    ///     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
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
    ///     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Box::new(move |event: &Event| -> Result<(), SyncDispatcherRequest> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.lock().test_method(&event);
    ///             Ok(())
    ///         } else {
    ///             Err(SyncDispatcherRequest::StopListening)
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
    /// and `Err(SyncDispatcherRequest::StopListening)` will cause them to
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
pub struct PriorityEventDispatcher<P, T>
where
    P: Ord,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: PriorityListenerMap<P, T>,
}

impl<P, T> Default for PriorityEventDispatcher<P, T>
where
    P: Ord + Clone,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    fn default() -> PriorityEventDispatcher<P, T> {
        PriorityEventDispatcher {
            events: PriorityListenerMap::new(),
        }
    }
}

impl<P, T> PriorityEventDispatcher<P, T>
where
    P: Ord + Clone,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
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
    ///     let mut dispatcher: PriorityEventDispatcher<u32, Event> = PriorityEventDispatcher::default();
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
    ///     let mut dispatcher: PriorityEventDispatcher<u32, Event> = PriorityEventDispatcher::default();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Box::new(move |event: &Event| -> Result<(), SyncDispatcherRequest> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.lock().test_method(&event);
    ///             Ok(())
    ///         } else {
    ///             Err(SyncDispatcherRequest::StopListening)
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
    /// and `Err(SyncDispatcherRequest::StopListening)` will cause them to
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

/// Errors for ThreadPool-building related failures.
#[derive(Fail, Debug)]
pub enum BuildError {
    #[fail(display = "Internal error on trying to build thread-pool: {:?}", _0)] NumThreads(String),
}

/// Owns a map of all listened event-variants,
/// [`Weak`]-references to their listeners and [`Fn`]s.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
pub struct ParallelEventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: ParallelListenerMap<T>,
    thread_pool: Option<ThreadPool>,
}

impl<T> Default for ParallelEventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static {
    fn default() -> ParallelEventDispatcher<T> {
        ParallelEventDispatcher {
            events: ParallelListenerMap::new(),
            thread_pool: None,
        }
    }
}

impl<T> ParallelEventDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// Adds a [`ParallelListener`] to listen for an `event_identifier`.
    /// If `event_identifier` is a new [`HashMap`]-key, it will be added.
    ///
    /// **Note**: If your `Enum` owns fields you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore fields,
    /// see second example for an implementation-suggestion.
    ///
    /// # Examples
    ///
    /// Adding a [`ParallelListener`] to the dispatcher:
    ///
    /// ```rust
    /// extern crate hey_listen;
    /// extern crate parking_lot;
    /// use std::sync::Arc;
    ///
    /// use hey_listen::ParallelListener;
    /// use hey_listen::ParallelEventDispatcher;
    /// use hey_listen::ParallelDispatcherRequest;
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// impl ParallelListener<Event> for ListenerStruct {
    ///     fn on_event(&mut self, event: &Event) -> Option<ParallelDispatcherRequest> { None }
    /// }
    ///
    /// fn main() {
    ///     let listener = Arc::new(parking_lot::Mutex::new(ListenerStruct {}));
    ///     let mut dispatcher: ParallelEventDispatcher<Event> = ParallelEventDispatcher::default();
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
    /// [`ParallelListener`]: trait.ParallelListener.html
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
    /// **Note**: If your `Enum` owns fields, you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore these.
    ///
    /// # Examples
    ///
    /// Adding a [`Fn`] to the dispatcher:
    ///
    /// ```rust
    /// extern crate hey_listen;
    /// extern crate parking_lot;
    /// extern crate failure;
    /// #[macro_use]
    /// extern crate failure_derive;
    ///
    /// use hey_listen::ParallelEventDispatcher;
    /// use hey_listen::ParallelDispatcherRequest;
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
    ///     let mut dispatcher: ParallelEventDispatcher<Event> = ParallelEventDispatcher::default();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Arc::new(move |event: &Event| -> Option<ParallelDispatcherRequest> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.lock().test_method(&event);
    ///             None
    ///         } else {
    ///             Some(ParallelDispatcherRequest::StopListening)
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

    /// Immediately after calling this method,
    /// the dispatcher will attempt to build a thread-pool with
    /// `num` amount of threads.
    /// If internals fail to build, [`BuildError`] is returned.
    ///
    /// **Note**: Failing to build the thread-pool will result
    /// in keeping the prior thread-pool, if one has been built before.
    /// If none has been built, none will be used; being default.
    ///
    /// [`BuildError`]: enum.BuildError.html
    pub fn num_threads(&mut self, num: usize) -> Result<(), BuildError> {
        match rayon::ThreadPoolBuilder::new().num_threads(num).build() {
            Ok(pool) => {
                self.thread_pool = Some(pool);
                Ok(())
            }
            Err(error) => Err(BuildError::NumThreads(error.description().to_string())),
        }
    }

    /// All [`ParallelListener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`Fn`]s returning an [`Option`] wrapping [`ParallelDispatcherRequest`]
    /// with `ParallelDispatcherRequest::StopListening` will cause them
    /// to be removed from the event-dispatcher.
    ///
    /// [`ParallelListener`]: trait.ParallelListener.html
    /// [`on_event`]: trait.ParallelListener.html#tymethod.on_event
    /// [`ParallelDispatcherRequest`]: enum.ParallelDispatcherRequest.html
    /// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
    /// [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(listener_collection) = self.events.get_mut(event_identifier) {
            let mut fns_to_remove = Mutex::new(Vec::new());
            let mut traits_to_remove = Mutex::new(Vec::new());

            if let Some(ref thread_pool) = self.thread_pool {
                thread_pool.install(|| {
                    ParallelEventDispatcher::joined_parallel_dispatch(
                        listener_collection,
                        event_identifier,
                        &fns_to_remove,
                        &traits_to_remove,
                    )
                });
            } else {
                ParallelEventDispatcher::joined_parallel_dispatch(
                    listener_collection,
                    event_identifier,
                    &fns_to_remove,
                    &traits_to_remove,
                );
            }

            fns_to_remove.lock().iter().for_each(|index| {
                listener_collection.fns.swap_remove(*index);
            });

            traits_to_remove.lock().iter().for_each(|index| {
                listener_collection.traits.swap_remove(*index);
            });
        }
    }

    /// Encapsulates `Rayon`'s joined `par_iter`-function on
    /// `Fn`s and `ParallelListener`s.
    ///
    /// This enables it to be used captured inside a `ThreadPool`'s
    /// `install`-method but also bare as is - in case no
    /// `ThreadPool` is avail.
    fn joined_parallel_dispatch(
        listener_collection: &ParallelFnsAndTraits<T>,
        event_identifier: &T,
        fns_to_remove: &Mutex<Vec<usize>>,
        traits_to_remove: &Mutex<Vec<usize>>,
    ) {
        join(
            || {
                listener_collection
                    .traits
                    .par_iter()
                    .enumerate()
                    .for_each(|(index, listener)| {
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
                    })
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
    }
}
