[![ci-badge][]][ci] [![docs-badge][]][docs] [![rust 1.33+ badge]][rust 1.33+ link] [![crates.io version]][crates.io link]

# Hey! Listen!

`Hey_listen` is a collection of event-dispatchers aiming to suit all needs!\
Currently covering:
* Synchronous dispatcher
* Priority dispatcher
* Threadpool dispatcher

Whenever applicable, dispatchers have an `Rc` and `Arc` variant.

View the `examples`-folder on how to use each dispatcher.

Everyone is welcome to contribute, check out the [`CONTRIBUTING.md`](CONTRIBUTING.md) for further guidance.

# Example

Here is a quick example on how to use the event-dispatcher:

```rust
extern crate hey_listen;

use hey_listen::{EventDispatcher, Listener, Mutex, SyncDispatcherRequest};
use std::sync::Arc;

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    Variant,
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
    let mut dispatcher = EventDispatcher::<Event>::default();

    dispatcher.add_listener(Event::Variant, &listener);
    dispatcher.dispatch_event(&Event::Variant);
}

```

# Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hey_listen = "0.2"
```

and this to your crate's root (not needed in Rust 2018):

```rust,ignore
extern crate hey_listen;
```

[ci-badge]: https://img.shields.io/travis/Lakelezz/hey_listen.svg?style=flat-square&colorB=3fb732
[ci]: https://travis-ci.org/Lakelezz/hey_listen

[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square&colorB=32b6b7
[docs]: https://docs.rs/hey_listen

[rust 1.33+ badge]: https://img.shields.io/badge/rust-1.33+-93450a.svg?style=flat-square&colorB=ff9a0d
[rust 1.33+ link]: https://blog.rust-lang.org/2019/02/28/Rust-1.33.0.html

[crates.io link]: https://crates.io/crates/hey_listen
[crates.io version]: https://img.shields.io/crates/v/hey_listen.svg?style=flat-square&colorB=b73732
