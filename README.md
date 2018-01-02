[![ci-badge][]][ci] [![docs-badge][]][docs]

# Hey_listen

Hey_listen is an event-dispatcher for Structs, Enums and other types.

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
hey_listen = "0.1"
parking_lot = "^0.5"
```

and this to your crate's root:

```rust,ignore
extern crate hey_listen;
extern crate parking_lot;
```

[ci-badge]: https://travis-ci.org/Lakelezz/hey_listen.svg?branch=master
[ci]: https://travis-ci.org/Lakelezz/hey_listen
[docs-badge]: https://docs.rs/hey_listen/badge.svg?version=0.1.1
[docs]: https://docs.rs/hey_listen