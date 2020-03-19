use super::{
    execute_sync_dispatcher_requests, ExecuteRequestsResult, PriorityListener,
};
use std::{
    collections::{BTreeMap, HashMap, btree_map::Entry as BTreeMapEntry, hash_map::Entry as HashMapEntry},
    hash::Hash,
};

type EventListener<T> = Box<dyn PriorityListener<T> + Send + Sync + 'static>;
type PriorityListenerMap<P, T> = HashMap<T, BTreeMap<P, Vec<EventListener<T>>>>;

/// In charge of prioritised sync dispatching to all listeners.
/// Opposed to [`EventListener`], this structure utilises one [`BTreeMap`] per
/// event-type to order listeners by a given priority-level.
///
/// **Note**: Consider implementing your own [`Ord`]-trait, if you
/// want a different priority.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
/// [`Ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
/// [`EventListener`]: struct.Dispatcher.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
pub struct PriorityDispatcher<P, T>
where
    P: Ord,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: PriorityListenerMap<P, T>,
}

impl<P, T> Default for PriorityDispatcher<P, T>
where
    P: Ord + Clone,
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    fn default() -> PriorityDispatcher<P, T> {
        PriorityDispatcher {
            events: PriorityListenerMap::new(),
        }
    }
}

impl<P, T> PriorityDispatcher<P, T>
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
    /// use std::sync::Arc;
    /// use hey_listen::{
    ///    RwLock,
    ///    sync::{PriorityListener, PriorityDispatcher, PriorityDispatcherRequest},
    /// };
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct;
    ///
    /// impl PriorityListener<Event> for ListenerStruct {
    ///     fn on_event(&self, event: &Event) -> Option<PriorityDispatcherRequest> { None }
    /// }
    ///
    /// fn main() {
    ///     let listener = ListenerStruct;
    ///     let mut dispatcher: PriorityDispatcher<u32, Event> = PriorityDispatcher::default();
    ///
    ///     dispatcher.add_listener(Event::EventType, listener, 1);
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
    /// [`Dispatcher`]: struct.Dispatcher.html
    /// [`Listener`]: trait.Listener.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    /// [`Ord`]: https://doc.rust-lang.org/std/cmp/trait.Ord.html
    pub fn add_listener<D: PriorityListener<T> + Send + Sync + 'static>(
        &mut self,
        event_key: T,
        listener: D,
        priority: P,
    ) {
        let listener = Box::new(listener);
        let listener = listener as Box<(dyn PriorityListener<T> + Send + Sync + 'static)>;


        match self.events.entry(event_key) {
            HashMapEntry::Vacant(vacant_entry) => {
                let mut map = BTreeMap::new();

                map.insert(
                    priority,
                    vec![listener]
                );

                vacant_entry.insert(map);
            },
            HashMapEntry::Occupied(mut occupied_entry) => {
                match occupied_entry.get_mut().entry(priority) {
                    BTreeMapEntry::Vacant(vacant_entry) => {
                        vacant_entry.insert(vec![listener]);
                    },
                    BTreeMapEntry::Occupied(mut occupied_entry) => {
                        occupied_entry.get_mut().push(listener);
                    },
                }
            },
        }
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

            for (_, mut listener_collection) in prioritised_listener_collection.iter_mut() {

                if let ExecuteRequestsResult::Stopped = execute_sync_dispatcher_requests(
                    &mut listener_collection,
                    |listener| {
                        listener.on_event(event_identifier)
                    },
                ) {
                    break;
                }
            }
        }
    }
}
