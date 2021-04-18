use super::{super::{Error, Mutex}, AsyncDispatchResult, AsyncListener};
use futures::{StreamExt, stream::FuturesUnordered};
use std::{collections::HashMap, hash::Hash};


/// In charge of parallel dispatching to all listeners.
pub struct AsyncDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: HashMap<T, Vec<Box<dyn AsyncListener<T> + Send + Sync + 'static>>>,
}

impl<T> AsyncDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sized + Sync + 'static,
{
    /// Create a new async dispatcher.
    /// Amount of threads must be set via Tokio.
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            events: HashMap::new(),
        })
    }

    /// Adds a [`AsyncListener`] to listen for an `event_key`.
    ///
    /// **Note**: If your `Enum` owns fields you need to consider implementing
    /// the [`Hash`]- and [`PartialEq`]-trait if you want to ignore fields,
    /// see second example for an implementation-suggestion.
    ///
    /// # Examples
    ///
    /// Adding a [`AsyncListener`] to the dispatcher:
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use hey_listen::{
    ///    RwLock,
    ///    sync::{AsyncListener, AsyncDispatcher, AsyncDispatchResult},
    /// };
    /// use async_trait::async_trait;
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// #[async_trait]
    /// impl AsyncListener<Event> for Arc<RwLock<ListenerStruct>> {
    ///     async fn on_event(&self, event: &Event) -> Option<AsyncDispatchResult> { None }
    /// }
    ///
    ///
    /// let listener = Arc::new(RwLock::new(ListenerStruct {}));
    /// let mut dispatcher: AsyncDispatcher<Event> = AsyncDispatcher::new()
    ///     .expect("Failed to build threadpool");
    ///
    /// dispatcher.add_listener(Event::EventType, Arc::clone(&listener));
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
    /// [`AsyncListener`]: trait.AsyncListener.html
    /// [`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
    /// [`PartialEq`]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
    pub fn add_listener<D: AsyncListener<T> + Send + Sync + Sized + 'static>(
        &mut self,
        event_key: T,
        listener: D,
    ) {
        let listener = Box::new(listener);

        self.events
            .entry(event_key)
            .or_insert_with(|| Vec::new())
            .push(listener as Box<(dyn AsyncListener<T> + Send + Sync + 'static)>);
    }

    /// All [`AsyncListener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`AsyncListener`]s returning an [`Option`] wrapping [`AsyncDispatchResult`]
    /// with `AsyncDispatchResult::StopListening` will cause them
    /// to be removed from the event-dispatcher.
    ///
    /// [`AsyncListener`]: trait.AsyncListener.html
    /// [`on_event`]: trait.AsyncListener.html#tymethod.on_event
    /// [`AsyncDispatchResult`]: enum.AsyncDispatchResult.html
    /// [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
    pub async fn dispatch_event<'a>(&mut self, event_identifier: &T) {
        if let Some(listeners) = self.events.get_mut(event_identifier) {
            let unordered_fut: FuturesUnordered<_> = FuturesUnordered::new();

            for (id, listener) in listeners.iter().enumerate() {
                let item = async move {
                    (id, listener.on_event(&event_identifier).await)
                };

                unordered_fut.push(item);
            }

            let listeners_to_remove = Mutex::new(Vec::<usize>::new());

            unordered_fut.for_each(|v| {
                if let Some(AsyncDispatchResult::StopListening) = v.1 {
                    listeners_to_remove.lock().push(v.0);
                }

                futures::future::ready(())
            }).await;

            listeners_to_remove.lock().iter().for_each(|index| {
                listeners.swap_remove(*index);
            });
        }
    }
}
