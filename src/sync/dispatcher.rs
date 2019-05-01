use super::{
    execute_sync_dispatcher_requests, FnsAndTraits, Listener, ListenerMap, RwLock,
    SyncDispatcherRequest,
};
use std::{
    hash::Hash,
    sync::{Arc, Weak},
};

/// In charge of sync dispatching to all listeners.
/// Owns a map event-variants and
/// [`Weak`]-references to their listeners and/or owns [`Fn`]s.
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
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
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
    /// use std::sync::Arc;
    ///
    /// use hey_listen::{Listener, EventDispatcher, RwLock, SyncDispatcherRequest};
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
    ///     let listener = Arc::new(RwLock::new(ListenerStruct {}));
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
    pub fn add_listener<D: Listener<T> + Send + Sync + 'static>(
        &mut self,
        event_identifier: T,
        listener: &Arc<RwLock<D>>,
    ) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.traits.push(Arc::downgrade(
                &(Arc::clone(listener) as Arc<RwLock<dyn Listener<T> + Send + Sync + 'static>>),
            ));

            return;
        }

        self.events.insert(
            event_identifier,
            FnsAndTraits::new_with_traits(vec![Arc::downgrade(
                &(Arc::clone(listener) as Arc<RwLock<dyn Listener<T> + Send + Sync + 'static>>),
            )]),
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
    /// use hey_listen::{EventDispatcher, RwLock, SyncDispatcherRequest};
    /// use std::sync::Arc;
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
    ///     let listener = Arc::new(RwLock::new(EventListener { used_method: false }));
    ///     let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Box::new(move |event: &Event| -> Option<SyncDispatcherRequest> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.write().test_method(&event);
    ///
    ///             None
    ///         } else {
    ///             Some(SyncDispatcherRequest::StopListening)
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
        function: Box<dyn Fn(&T) -> Option<SyncDispatcherRequest> + Send + Sync + 'static>,
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

            execute_sync_dispatcher_requests(&mut listener_collection.traits, |weak_listener| {
                if let Some(listener_arc) = weak_listener.upgrade() {
                    let mut listener = listener_arc.write();
                    listener.on_event(event_identifier)
                } else {
                    found_invalid_weak_ref = true;
                    None
                }
            });

            execute_sync_dispatcher_requests(&mut listener_collection.fns, |callback| {
                callback(event_identifier)
            });

            if found_invalid_weak_ref {
                listener_collection
                    .traits
                    .retain(|listener| Weak::clone(listener).upgrade().is_some());
            }
        }
    }
}
