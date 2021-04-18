use std::hash::Hash;

/// Contains the blocking dispatcher.
pub mod dispatcher;

/// Puts the blocking dispatcher in scope.
pub use dispatcher::Dispatcher;

/// Every event-receiver needs to implement this trait
/// in order to receive dispatched events.
/// `T` being the type you use for events, e.g. an `Enum`.
pub trait Listener<T>
where
    T: PartialEq + Eq + Hash + Clone + 'static,
{
    /// This function will be called once a listened
    /// event-type `T` has been dispatched.
    fn on_event(&self, event: &T) -> Option<DispatcherRequest>;
}

/// When `execute_sync_dispatcher_requests` returns,
/// this `enum` informs on whether the return is early
/// and thus forcefully stopped or finished on its own.
#[derive(Debug)]
pub(crate) enum ExecuteRequestsResult {
    Finished,
    Stopped,
}

/// An `enum` returning a request from a listener to the single-threaded dispatcher.
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
pub enum DispatcherRequest {
    /// Stops listening to the dispatcher.
    StopListening,
    /// Stops the event to be dispatched to other listeners.
    StopPropagation,
    /// Stops listening to the dispatcher and prevents the event from further
    /// dispatch.
    StopListeningAndPropagation,
}

/// Iterates over the passed `vec` and applies `function` to each element.
/// `function`'s returned [`SyncDispatchResult`] will instruct
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
/// can break the current iteration and is able to match [`SyncDispatchResult`]
/// and perform actions based on variants.
///
/// [`retain`]: https://doc.rust-lang.org/alloc/vec/struct.Vec.html#method.retain
/// [`SyncDispatchResult`]: enum.SyncDispatchResult.html
pub(crate) fn execute_dispatcher_requests<T, F>(
    vec: &mut Vec<T>,
    mut function: F,
) -> ExecuteRequestsResult
where
    F: FnMut(&T) -> Option<DispatcherRequest>,
{
    let mut index = 0;

    loop {
        if index < vec.len() {
            match function(&vec[index]) {
                None => index += 1,
                Some(DispatcherRequest::StopListening) => {
                    vec.swap_remove(index);
                }
                Some(DispatcherRequest::StopPropagation) => return ExecuteRequestsResult::Stopped,
                Some(DispatcherRequest::StopListeningAndPropagation) => {
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

        fn map_usize_to_request(x: &usize) -> Option<DispatcherRequest> {
            match *x {
                0 => Some(DispatcherRequest::StopListening),
                1 => Some(DispatcherRequest::StopPropagation),
                2 => Some(DispatcherRequest::StopListeningAndPropagation),
                _ => None,
            }
        }

        #[test]
        fn stop_listening() {
            let mut vec = vec![0, 0, 0, 1, 1, 1, 1];
            execute_dispatcher_requests(&mut vec, map_usize_to_request);

            assert_eq!(vec, [1, 0, 0, 1, 1, 1]);
        }

        #[test]
        fn empty_vec() {
            let mut vec = Vec::new();
            execute_dispatcher_requests(&mut vec, map_usize_to_request);

            assert!(vec.is_empty());
        }

        #[test]
        fn removing_all() {
            let mut vec = vec![0, 0, 0, 0, 0, 0, 0];
            execute_dispatcher_requests(&mut vec, map_usize_to_request);

            assert!(vec.is_empty());
        }

        #[test]
        fn remove_one_element_and_stop() {
            let mut vec = vec![2, 0];
            execute_dispatcher_requests(&mut vec, map_usize_to_request);

            assert_eq!(vec, [0]);
        }
    }
}
