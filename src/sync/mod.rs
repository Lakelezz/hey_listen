#[cfg(feature = "parallel")]
use rayon::ThreadPool;
use std::hash::Hash;

#[cfg(feature = "parallel")]
/// This module contains the parallel dispatcher.
pub mod parallel_dispatcher;
#[cfg(feature = "parallel")]
/// This module contains the priority dispatcher.
pub mod priority_dispatcher;
#[cfg(feature = "async")]
/// This module contains the async dispatcher.
pub mod async_dispatcher;

#[cfg(feature = "parallel")]
pub use parallel_dispatcher::ParallelDispatcher;
#[cfg(feature = "parallel")]
pub use priority_dispatcher::PriorityDispatcher;
#[cfg(feature = "async")]
pub use async_dispatcher::AsyncDispatcher;

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
#[cfg(feature = "parallel")]
#[derive(Debug)]
// Clippy complains that all variants have the same prefix.
// However, the term `Stop` is an essential component for the meaning of each
// variant's name.
#[allow(clippy::pub_enum_variant_names)]
pub enum PriorityDispatcherResult {
    /// Stops listening to the dispatcher.
    StopListening,
    /// Prevents the event from reaching the next less important priority group.
    StopPropagation,
    /// Stops listening and prevents events from reaching the next less
    /// priority group.
    StopListeningAndPropagation,
}

/// When `execute_sync_dispatcher_requests` returns,
/// this `enum` informs on whether the return is early
/// and thus forcefully stopped or finished on its own.
#[derive(Debug)]
#[cfg(feature = "parallel")]
pub(crate) enum ExecuteRequestsResult {
    Finished,
    Stopped,
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
#[cfg(feature = "parallel")]
pub trait Listener<T>
where
    T: PartialEq + Eq + Hash + Clone + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&self, event: &T) -> Option<ParallelDispatchResult>;
}

/// Iterates over the passed `vec` and applies `function` to each element.
/// `function`'s returned [`ParallelDispatchResult`] will instruct
/// a procedure depending on its variant:
///
/// `StopListening`: Removes item from `vec`.
/// `StopPropagation`: Stops further dispatching to other elements
/// in `vec`.
/// `StopListeningAndPropagation`: Execute `StopListening`,
/// then execute `StopPropagation`.
///
/// **Note**: When `StopListening` is being executed,
/// removal of items from `vec` will swap elements,
/// resulting in a different order.
///
/// **Note**: Unlike [`retain`], `execute_sync_dispatcher_requests`
/// can stop the current iteration and is able to match [`ParallelDispatchResult`]
/// and perform actions based on variants.
///
/// [`retain`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html#method.retain
/// [`ParallelDispatchResult`]: enum.ParallelDispatchResult.html
#[cfg(feature = "parallel")]
pub(crate) fn execute_sync_dispatcher_requests<T, F>(
    vec: &mut Vec<T>,
    mut function: F,
) -> ExecuteRequestsResult
where
    F: FnMut(&T) -> Option<PriorityDispatcherResult>,
{
    let mut index = 0;

    loop {
        if index < vec.len() {
            match function(&vec[index]) {
                None => index += 1,
                Some(PriorityDispatcherResult::StopListening) => {
                    vec.swap_remove(index);
                }
                Some(PriorityDispatcherResult::StopPropagation) => {
                    return ExecuteRequestsResult::Stopped
                }
                Some(PriorityDispatcherResult::StopListeningAndPropagation) => {
                    vec.swap_remove(index);
                    return ExecuteRequestsResult::Stopped;
                }
            }
        } else {
            return ExecuteRequestsResult::Finished;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod execute_sync_dispatcher_requests {
        use super::*;

        fn map_usize_to_request(x: &usize) -> Option<PriorityDispatcherResult> {
            match *x {
                0 => Some(PriorityDispatcherResult::StopListening),
                1 => Some(PriorityDispatcherResult::StopPropagation),
                2 => Some(PriorityDispatcherResult::StopListeningAndPropagation),
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
/// Opposed to `ParallelDispatchResult` a [`Listener`] cannot
/// stop propagation as the propagation is happening in parallel.
///
/// [`Listener`]: trait.Listener.html
#[cfg(feature = "parallel")]
#[derive(Debug)]
pub enum ParallelDispatchResult {
    /// Stops the listener from receiving further events from the dispatcher.
    StopListening,
}

/// An `enum` returning a request from a [`Listener`] to its async event-dispatcher.
///
/// `StopListening` will remove your [`Listener`] from the
/// event-dispatcher.
#[derive(Debug)]
#[cfg(feature = "async")]
pub enum AsyncDispatchResult {
    /// Stops the listener from receiving further events from the dispatcher.
    StopListening,
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncListener<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    /// If you want to mutate the listener, consider wrapping it behind an
    /// `RwLock` or `Mutex`.
    async fn on_event(&self, event: &T) -> Option<AsyncDispatchResult>;
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
#[cfg(feature = "parallel")]
pub trait ParallelListener<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    /// If you want to mutate the listener, consider wrapping it behind an
    /// `RwLock` or `Mutex`.
    fn on_event(&self, event: &T) -> Option<ParallelDispatchResult>;
}

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
#[cfg(feature = "parallel")]
pub trait PriorityListener<T>
where
    T: PartialEq + Eq + Hash + Clone + Send + Sync + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    /// If you want to mutate the listener, consider wrapping it behind an
    /// `RwLock` or `Mutex`.
    fn on_event(&self, event: &T) -> Option<PriorityDispatcherResult>;
}
