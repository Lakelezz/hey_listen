# Change Log

Covering up all the hot and cold changes!

## [0.2.0]

This release adds a parallel dispatcher and allows listeners to return requests back to their dispatcher.
These requests have following effects: Stopping the event propagation, unsubscribing from further dispatch, and combining both concepts.
Nonetheless, requests to parallel dispatchers are limited to unsubscribing.

### Added

- `ParallelEventDispatcher` has been added.
- `SyncDispatcherRequest` to return instructions from listeners back to dispatcher.
- `ParallelDispatcherRequest` to return instructions from listeners back to parallel dispatcher.
- Examples have been added.

### Breaking Changes

- Synchronous dispatchers return `Option<SyncDispatcherRequest>`.

## [0.1.2]

This release supports callbacking closures and ordering dispatching via priority-levels.
