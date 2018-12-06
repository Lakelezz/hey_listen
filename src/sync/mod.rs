use super::Mutex;
use rayon::ThreadPool;
use std::{collections::HashMap, hash::Hash, sync::Weak};

pub mod dispatcher;
pub mod parallel_dispatcher;
pub mod priority_dispatcher;

type EventFunction<T> = Vec<Box<Fn(&T) -> Option<SyncDispatcherRequest> + Send + Sync>>;
type ListenerMap<T> = HashMap<T, FnsAndTraits<T>>;

type ParallelListenerMap<T> = HashMap<T, ParallelFnsAndTraits<T>>;
type ParallelEventFunction<T> = Vec<Box<Fn(&T) -> Option<ParallelDispatcherRequest> + Send + Sync>>;

/// An `enum` returning a request from a listener to its `sync` event-dispatcher.
/// This `enum` is not restricted to dispatcher residing in the `sync`-module.
/// A request will be processed by the event-dispatcher depending on the variant:
///
/// `StopListening` will remove your listener from the event-dispatcher.
///
/// `StopPropagation` will stop dispatching of the current `Event` instance.
/// Therefore, a listener issuing this is the last receiver.
///
/// `StopListeningAndPropagation` a combination of first `StopListening`
/// and then `StopPropagation`.
#[derive(Debug)]
pub enum SyncDispatcherRequest {
    StopListening,
    StopPropagation,
    StopListeningAndPropagation,
}

/// When `execute_sync_dispatcher_requests` returns,
/// this `enum` informs on whether the return is early
/// and thus forcefully stopped or finished on its own.
#[derive(Debug)]
pub(crate) enum ExecuteRequestsResult {
    Finished,
    Stopped,
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
pub trait Listener<T>
where
    T: PartialEq + Eq + Hash + Clone + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&mut self, event: &T) -> Option<SyncDispatcherRequest>;
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
/// **Note**: Unlike [`retain`], `execute_sync_dispatcher_requests`
/// can break the current iteration and is able to match [`SyncDispatcherRequest`]
/// and perform actions based on variants.
///
/// [`retain`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html#method.retain
/// [`SyncDispatcherRequest`]: enum.SyncDispatcherRequest.html
pub(crate) fn execute_sync_dispatcher_requests<T, F>(
    vec: &mut Vec<T>,
    mut function: F,
) -> ExecuteRequestsResult
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
                Some(SyncDispatcherRequest::StopPropagation) => {
                    return ExecuteRequestsResult::Stopped
                }
                Some(SyncDispatcherRequest::StopListeningAndPropagation) => {
                    vec.swap_remove(index);
                    return ExecuteRequestsResult::Stopped;
                }
            }
        } else {
            return ExecuteRequestsResult::Finished;
        }
    }
}

/// Yields closures and trait-objects.
struct FnsAndTraits<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    traits: Vec<Weak<Mutex<Listener<T> + Send + Sync + 'static>>>,
    fns: EventFunction<T>,
}

impl<T> FnsAndTraits<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    fn new_with_traits(
        trait_objects: Vec<Weak<Mutex<Listener<T> + Send + Sync + 'static>>>,
    ) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod execute_sync_dispatcher_requests {
        use super::*;

        fn map_usize_to_request(x: &usize) -> Option<SyncDispatcherRequest> {
            match *x {
                0 => Some(SyncDispatcherRequest::StopListening),
                1 => Some(SyncDispatcherRequest::StopPropagation),
                2 => Some(SyncDispatcherRequest::StopListeningAndPropagation),
                _ => None,
            }
        }

        #[test]
        fn stop_listening() {
            let mut vec = vec![0, 0, 0, 1, 1, 1, 1];
            execute_sync_dispatcher_requests(&mut vec, map_usize_to_request);

            assert_eq!(vec, [1, 0, 0, 1, 1, 1]);
        }

        #[test]
        fn empty_vec() {
            let mut vec = Vec::new();
            execute_sync_dispatcher_requests(&mut vec, map_usize_to_request);

            assert!(vec.is_empty());
        }

        #[test]
        fn removing_all() {
            let mut vec = vec![0, 0, 0, 0, 0, 0, 0];
            execute_sync_dispatcher_requests(&mut vec, map_usize_to_request);

            assert!(vec.is_empty());
        }

        #[test]
        fn remove_one_element_and_stop() {
            let mut vec = vec![2, 0];
            execute_sync_dispatcher_requests(&mut vec, map_usize_to_request);

            assert_eq!(vec, [0]);
        }
    }
}

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
