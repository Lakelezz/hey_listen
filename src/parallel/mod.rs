pub mod dispatcher;

use super::Mutex;
use rayon::ThreadPool;
use std::{collections::HashMap, hash::Hash, sync::Weak};

type ParallelListenerMap<T> = HashMap<T, ParallelFnsAndTraits<T>>;
type ParallelEventFunction<T> = Vec<Box<Fn(&T) -> Option<ParallelDispatcherRequest> + Send + Sync>>;

/// An `enum` returning a request from a [`Listener`] to its parallel event-dispatcher.
///
/// `StopListening` will remove your [`Listener`] from the
/// event-dispatcher.
///
/// **Note**:
/// Opposed to `SyncDispatcherRequest` a [`Listener`] cannot
/// stop propagation as the propagation is happening parallel.
///
/// [`Listener`]: trait.Listener.html
#[derive(Debug)]
pub enum ParallelDispatcherRequest {
    StopListening,
}

/// Yields `Send` and `Sync` closures and trait-objects.
struct ParallelFnsAndTraits<T>
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
pub trait ParallelListener<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&mut self, event: &T) -> Option<ParallelDispatcherRequest>;
}

/// Errors for ThreadPool-building related failures.
#[derive(Fail, Debug)]
pub enum BuildError {
    #[fail(
        display = "Internal error on trying to build thread-pool: {:?}",
        _0
    )]
    NumThreads(String),
}
