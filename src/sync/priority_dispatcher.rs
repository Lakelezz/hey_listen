use super::{
    execute_sync_dispatcher_requests, ExecuteRequestsResult, FnsAndTraits, Listener,
    SyncDispatcherRequest,
};
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Weak};

type PriorityListenerMap<P, T> = HashMap<T, BTreeMap<P, FnsAndTraits<T>>>;

/// In charge of prioritised sync dispatching to all listeners.
/// Owns a map event-variants and [`Weak`]-references to their
/// listeners and/or owns [`Fn`]s.
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
    /// use hey_listen::{Listener, PriorityEventDispatcher, SyncDispatcherRequest};
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// impl Listener<Event> for ListenerStruct {
    ///     fn on_event(&mut self, event: &Event) -> Option<SyncDispatcherRequest> { None }
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
                    &(Arc::clone(listener) as Arc<Mutex<Listener<T> + Send + Sync + 'static>>),
                ));

                return;
            }
            prioritised_listener_collection.insert(
                priority.clone(),
                FnsAndTraits::new_with_traits(vec![Arc::downgrade(
                    &(Arc::clone(listener) as Arc<Mutex<Listener<T> + Send + Sync + 'static>>),
                )]),
            );
            return;
        }

        let mut b_tree_map = BTreeMap::new();
        b_tree_map.insert(
            priority,
            FnsAndTraits::new_with_traits(vec![Arc::downgrade(
                &(Arc::clone(listener) as Arc<Mutex<Listener<T> + Send + Sync + 'static>>),
            )]),
        );
        self.events.insert(event_identifier, b_tree_map);
    }

    /// Adds an [`Fn`] to listen for an `event_identifier`, considering
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
    /// use hey_listen::{PriorityEventDispatcher, SyncDispatcherRequest};
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
    ///     let closure = Box::new(move |event: &Event| -> Option<SyncDispatcherRequest> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.lock().test_method(&event);
    ///
    ///             None
    ///         } else {
    ///             Some(SyncDispatcherRequest::StopListening)
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
        function: Box<Fn(&T) -> Option<SyncDispatcherRequest> + Send + Sync>,
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

                if let ExecuteRequestsResult::Stopped = execute_sync_dispatcher_requests(
                    &mut listener_collection.traits,
                    |weak_listener| {
                        if let Some(listener_arc) = weak_listener.upgrade() {
                            let mut listener = listener_arc.lock();
                            listener.on_event(event_identifier)
                        } else {
                            found_invalid_weak_ref = true;
                            None
                        }
                    },
                ) {
                    break;
                }

                if let ExecuteRequestsResult::Stopped =
                    execute_sync_dispatcher_requests(&mut listener_collection.fns, |callback| {
                        callback(event_identifier)
                    }) {
                    break;
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
