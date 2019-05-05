# Change Log

Covering up all the changes!

## [0.4.0]

Just a `parking_lot`-dependency update to `0.8`.

### Breaking Changes

- `parking_lot` updated to `0.8`.

## [0.3.0]

This release updates the `parking_lot`-dependency to `0.7` and while we are at
it: From now on, `hey_listen` will use `RwLock` for the listeners instead of
`Mutex`.

On another note, the term `EventDispatcher` has been simplified
to just `Dispatcher` as every dispatcher in this library dispatches events.

Also, the re-exports added in `0.2.1`, which warranted stability,
have been removed.\

Last and probably least important: The source code has been updated to
Rust 2018. Somewhat related to that, the new required Rust version is `1.34.1`.

### Breaking Changes

- All `add_listener`-methods expect a listener wrapped inside a
`parking_lot::RwLock` instead of a `parking_lot::Mutex`.
- `parking_lot` updated to `0.7`.
- Re-exports introduced in `0.2.1` have been removed.
- New required Rust version: `1.34.1`.

---

## [0.2.1]

This release adds `std::rc::Rc`-alternatives for dispatchers that do not require to be `Send` and `Sync`.\
Furthermore, `hey_listen` is now refactored into two modules:
 * `rc`: dispatchers using `std::rc::Rc`.
 * `sync`: dispatchers using `std::sync::Arc`.

Nonetheless, we re-import everything into the crate's root securing stability.

Optionally, consider updating imports:
 - `hey_listen::sync::Dispatcher` instead of `hey_listen::Dispatcher`.
 - `hey_listen::sync::PriorityDispatcher` instead of `hey_listen::PriorityDispatcher`.
 - `hey_listen::sync:ParallelDispatcher` instead of `hey_listen::ParallelDispatcher`.
 - `hey_listen::sync::ParallelListener` instead of `hey_listen::ParallelListener`.

### Added

- `hey_listen::rc::Dispatcher` to use `Rc` instead of `Arc`.
- `hey_listen::rc::PriorityDispatcher` to use `Rc` instead of `Arc`.
- `sync` and `rc` modules.
- `parking_lot::Mutex` is now re-imported and can be accessed via `hey_listen::Mutex`.

---

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

---

## [0.1.2]

This release supports callbacking closures and ordering dispatching via priority-levels.
