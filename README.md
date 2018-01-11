[![ci-badge][]][ci] [![docs-badge][]][docs]

# Hey! Listen!

Hey_listen is an event-dispatcher for Closures, Structs, Enums and other types.
On another note, hey_listen supports prioritised/ordered dispatching and aims to add
parallel dispatching as well!

Everyone is welcome to contribute, check out the [`CONTRIBUTING.md`](CONTRIBUTING.md) for further guidance.

# Example

Here is a quick example on how to use the event-dispatcher:

```rust
extern crate hey_listen;
extern crate parking_lot;

use hey_listen::Listener;
use hey_listen::EventDispatcher;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone, Eq, Hash, PartialEq)]
enum EventEnum {
    EventVariant,
}

struct ListenerStruct {}

impl Listener<EventEnum> for ListenerStruct {
    fn on_event(&mut self, event: &EventEnum) {
        println!("I'm listening! :)");
    }
}

fn main() {
    let listener = Arc::new(Mutex::new(ListenerStruct {}));
    let mut dispatcher: EventDispatcher<EventEnum> = EventDispatcher::new();

    dispatcher.add_listener(EventEnum::EventVariant, &listener);
}
```

# Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hey_listen = "0.1.2"
parking_lot = "^0.5"
```

and this to your crate's root:

```rust,ignore
extern crate hey_listen;
extern crate parking_lot;
```

[ci-badge]: https://travis-ci.org/Lakelezz/hey_listen.svg?branch=master
[ci]: https://travis-ci.org/Lakelezz/hey_listen
[docs-badge]: https://docs.rs/hey_listen/badge.svg?version=0.1.2
[docs]: https://docs.rs/hey_listen