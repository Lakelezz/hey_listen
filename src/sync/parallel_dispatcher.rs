use super::{
    super::RwLock, BuildError, ParallelDispatcherRequest, ParallelFnsAndTraits, ParallelListener,
    ParallelListenerMap, ThreadPool,
};
use rayon::{
    join,
    prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator},
    ThreadPoolBuilder,
};
use std::{error::Error, hash::Hash, sync::Arc};

/// In charge of parallel dispatching to all listeners.
/// Owns a map event-variants and [`Weak`]-references to their listeners
/// and/or owns [`Fn`]s.
///
/// [`Weak`]: https://doc.rust-lang.org/std/sync/struct.Weak.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
pub struct ParallelDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    events: ParallelListenerMap<T>,
    thread_pool: Option<ThreadPool>,
}

impl<T> Default for ParallelDispatcher<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    fn default() -> ParallelDispatcher<T> {
        ParallelDispatcher {
            events: ParallelListenerMap::new(),
            thread_pool: None,
        }
    }
}

impl<T> ParallelDispatcher<T>
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
    /// use std::sync::Arc;
    /// use hey_listen::{
    ///    RwLock,
    ///    sync::{ParallelListener, ParallelDispatcher, ParallelDispatcherRequest},
    /// };
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
    ///     let listener = Arc::new(RwLock::new(ListenerStruct {}));
    ///     let mut dispatcher: ParallelDispatcher<Event> = ParallelDispatcher::default();
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
        listener: &Arc<RwLock<D>>,
    ) {
        if let Some(listener_collection) = self.events.get_mut(&event_identifier) {
            listener_collection.traits.push(Arc::downgrade(
                &(Arc::clone(listener)
                    as Arc<RwLock<dyn ParallelListener<T> + Send + Sync + 'static>>),
            ));

            return;
        }

        self.events.insert(
            event_identifier,
            ParallelFnsAndTraits::new_with_traits(vec![Arc::downgrade(
                &(Arc::clone(listener)
                    as Arc<RwLock<dyn ParallelListener<T> + Send + Sync + 'static>>),
            )]),
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
    /// use failure_derive::Fail;
    ///
    /// use hey_listen::{
    ///    RwLock,
    ///    sync::{Listener, ParallelDispatcher, ParallelDispatcherRequest},
    /// };
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
    ///     let mut dispatcher: ParallelDispatcher<Event> = ParallelDispatcher::default();
    ///     let weak_listener_ref = Arc::downgrade(&Arc::clone(&listener));
    ///
    ///     let closure = Box::new(move |event: &Event| -> Option<ParallelDispatcherRequest> {
    ///         if let Some(listener) = weak_listener_ref.upgrade() {
    ///             listener.write().test_method(&event);
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
        function: Box<dyn Fn(&T) -> Option<ParallelDispatcherRequest> + Send + Sync>,
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
        match ThreadPoolBuilder::new().num_threads(num).build() {
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
            let fns_to_remove = RwLock::new(Vec::new());
            let traits_to_remove = RwLock::new(Vec::new());

            if let Some(ref thread_pool) = self.thread_pool {
                thread_pool.install(|| {
                    ParallelDispatcher::joined_parallel_dispatch(
                        listener_collection,
                        event_identifier,
                        &fns_to_remove,
                        &traits_to_remove,
                    )
                });
            } else {
                ParallelDispatcher::joined_parallel_dispatch(
                    listener_collection,
                    event_identifier,
                    &fns_to_remove,
                    &traits_to_remove,
                );
            }

            fns_to_remove.write().iter().for_each(|index| {
                listener_collection.fns.swap_remove(*index);
            });

            traits_to_remove.write().iter().for_each(|index| {
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
        fns_to_remove: &RwLock<Vec<usize>>,
        traits_to_remove: &RwLock<Vec<usize>>,
    ) {
        join(
            || {
                listener_collection
                    .traits
                    .par_iter()
                    .enumerate()
                    .for_each(|(index, listener)| {
                        if let Some(listener_arc) = listener.upgrade() {
                            let mut listener = listener_arc.write();

                            if let Some(instruction) = listener.on_event(event_identifier) {
                                match instruction {
                                    ParallelDispatcherRequest::StopListening => {
                                        traits_to_remove.write().push(index)
                                    }
                                }
                            }
                        } else {
                            traits_to_remove.write().push(index)
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
                                    fns_to_remove.write().push(index);
                                }
                            }
                        }
                    });
            },
        );
    }
}
