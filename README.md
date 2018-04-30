[![ci-badge][]][ci] [![docs-badge][]][docs]

# Hey! Listen!

`Hey_listen` is a collection of event-dispatchers aiming to suit all needs!

Covering synchronous, parallel, and sync-prioritised dispatching to Closures, Enums, Structs, and every other type supporting `trait`-implementation.

View the `examples`-folder on how to use each dispatcher.

Everyone is welcome to contribute, check out the [`CONTRIBUTING.md`](CONTRIBUTING.md) for further guidance.

# Example

Here is a quick example on how to use the event-dispatcher:

```rust
extern crate hey_listen;
extern crate parking_lot;

use hey_listen::{EventDispatcher, Listener, SyncDispatcherRequest};
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    EventVariant,
}

struct ListenerStruct {}

impl Listener<Event> for ListenerStruct {
    fn on_event(&mut self, _event: &Event) -> Option<SyncDispatcherRequest> {
        println!("I'm listening! :)");

        None
    }
}

fn main() {
    let listener = Arc::new(Mutex::new(ListenerStruct {}));
    let mut dispatcher: EventDispatcher<Event> = EventDispatcher::default();

    dispatcher.add_listener(Event::EventVariant, &listener);
    dispatcher.dispatch_event(&Event::EventVariant);
}

```

# Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hey_listen = "0.2.0"
parking_lot = "^0.5.4"
```

and this to your crate's root:

```rust,ignore
extern crate hey_listen;
extern crate parking_lot;
```

[ci-badge]: https://travis-ci.org/Lakelezz/hey_listen.svg?branch=master
[ci]: https://travis-ci.org/Lakelezz/hey_listen
[docs-badge]: https://docs.rs/hey_listen/badge.svg?version=0.2.0
[docs]: https://docs.rs/hey_listen
