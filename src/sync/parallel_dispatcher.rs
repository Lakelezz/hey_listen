use super::{
    super::{Error, Mutex},
    ParallelDispatchResult, ParallelListener, ThreadPool,
};
use rayon::{
    prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use std::{collections::HashMap, hash::Hash};

/// In charge of parallel dispatching to all listeners.
pub struct ParallelDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: HashMap<T, Vec<Box<dyn ParallelListener<T> + Send + Sync + 'static>>>,
    thread_pool: ThreadPool,
}

impl<T> ParallelDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sized + Sync + 'static,
{
    /// Creates a parallel dispatcher with `num_threads` amount of threads.
    ///
    /// # Errors
    /// Fails with [`Error::ThreadPoolBuilder`] when building the fails.
    pub fn new(num_threads: usize) -> Result<Self, Error> {
        Ok(Self {
            events: HashMap::new(),
            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()?,
        })
    }

    /// Adds a [`ParallelListener`] to listen for an `event_key`.
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
    /// use std::sync::Arc;
    /// use hey_listen::{
    ///    RwLock,
    ///    sync::{ParallelListener, ParallelDispatcher, ParallelDispatchResult},
    /// };
    ///
    /// #[derive(Clone, Eq, Hash, PartialEq)]
    /// enum Event {
    ///     EventType,
    /// }
    ///
    /// struct ListenerStruct {}
    ///
    /// impl ParallelListener<Event> for Arc<RwLock<ListenerStruct>> {
    ///     fn on_event(&self, event: &Event) -> Option<ParallelDispatchResult> { None }
    /// }
    ///
    ///
    /// let listener = Arc::new(RwLock::new(ListenerStruct {}));
    /// let mut dispatcher: ParallelDispatcher<Event> = ParallelDispatcher::new(1)
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
    /// [`ParallelListener`]: ParallelListener
    /// [`Hash`]: std::hash::Hash
    /// [`PartialEq`]: std::cmp::PartialEq
    pub fn add_listener<D: ParallelListener<T> + Send + Sync + Sized + 'static>(
        &mut self,
        event_key: T,
        listener: D,
    ) {
        let listener = Box::new(listener);

        self.events
            .entry(event_key)
            .or_insert_with(Vec::new)
            .push(listener as Box<(dyn ParallelListener<T> + Send + Sync + 'static)>);
    }

    /// Immediately after calling this method,
    /// the dispatcher will attempt to build a thread-pool with
    /// `num` amount of threads.
    ///
    ///
    /// # Errors
    /// If internals fail to build, [`Error::ThreadPoolBuilder`] is returned.
    /// Failing to build the thread-pool will result
    /// in keeping the prior thread-pool.
    ///
    ///
    /// [`Error::ThreadPoolBuilder`]: Error::ThreadPoolBuilder
    pub fn num_threads(&mut self, num: usize) -> Result<(), Error> {
        self.thread_pool = ThreadPoolBuilder::new().num_threads(num).build()?;

        Ok(())
    }

    /// All [`ParallelListener`]s listening to a passed `event_identifier`
    /// will be called via their implemented [`on_event`]-method.
    /// [`ParallelListener`]s returning an [`Option`] wrapping [`ParallelDispatchResult`]
    /// with `ParallelDispatchResult::StopListening` will cause them
    /// to be removed from the event-dispatcher.
    ///
    /// [`ParallelListener`]: ParallelListener
    /// [`on_event`]: ParallelListener::on_event
    /// [`ParallelDispatchResult`]: ParallelDispatchResult
    /// [`Option`]: std::option::Option
    pub fn dispatch_event(&mut self, event_identifier: &T) {
        if let Some(listener_collection) = self.events.get_mut(event_identifier) {
            let listeners_to_remove = Mutex::new(Vec::new());

            self.thread_pool.install(|| {
                listener_collection
                    .par_iter()
                    .enumerate()
                    .for_each(|(index, listener)| {
                        if let Some(instruction) = listener.on_event(event_identifier) {
                            match instruction {
                                ParallelDispatchResult::StopListening => {
                                    listeners_to_remove.lock().push(index)
                                }
                            }
                        }
                    })
            });

            listeners_to_remove.lock().iter().for_each(|index| {
                listener_collection.swap_remove(*index);
            });
        }
    }
}
