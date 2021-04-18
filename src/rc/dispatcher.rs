use super::{execute_dispatcher_requests, Listener};
use std::{collections::HashMap, hash::Hash};

/// In charge of parallel dispatching to all listeners.
pub struct Dispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + 'static,
{
    events: HashMap<T, Vec<Box<dyn Listener<T> + 'static>>>,
}

impl<T> Dispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Sized + 'static,
{
    /// Create a new blocking dispatcher.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// Adds a [`Listener`] to listen for an `event_key`.
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
    /// use hey_listen::rc::{
    ///     Listener, Dispatcher, DispatcherRequest,
    /// };
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct;
    ///
    /// impl Listener<Event> for ListenerStruct {
    ///     fn on_event(&self, event: &Event) -> Option<DispatcherRequest> { None }
    /// }
    ///
    /// let listener = ListenerStruct;
    /// let mut dispatcher: Dispatcher<Event> = Dispatcher::new();
    ///
    /// dispatcher.add_listener(Event::EventType, listener);
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
    pub fn add_listener<D: Listener<T> + Sized + 'static>(&mut self, event_key: T, listener: D) {
        let listener = Box::new(listener);

        self.events
            .entry(event_key)
            .or_insert_with(Vec::new)
            .push(listener as Box<dyn Listener<T> + 'static>);
    }

    /// All [`Listener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`Listener`]s returning an [`Option`] wrapping [`DispatcherRequest`]
    /// with `DispatcherRequest::StopListening` will cause them
    /// to be removed from the event-dispatcher.
    ///
    /// [`Listener`]: trait.Listener.html
    /// [`on_event`]: trait.Listener.html#tymethod.on_event
    /// [`DispatcherRequest`]: enum.DispatcherRequest.html
    /// [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(mut listener_collection) = self.events.get_mut(event_identifier) {
            execute_dispatcher_requests(&mut listener_collection, |listener| {
                listener.on_event(event_identifier)
            });
        }
    }
}
