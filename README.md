[![ci-badge][]][ci] [![docs-badge][]][docs] [![rust version badge]][rust version link] [![crates.io version]][crates.io link]

# Hey! Listen!

`Hey_listen` is a collection of event-dispatchers aiming to suit all needs!\
Currently covering:
* Priority dispatch
* Threadpool dispatch
* Async dispatch

View the `examples`-folder on how to use each dispatcher.

Everyone is welcome to contribute, check out the [`CONTRIBUTING.md`](CONTRIBUTING.md) for further guidance.

# Example

Here is a quick example on how to use the event-dispatcher:

```rust,no_run
use hey_listen::sync::{
    ParallelDispatcher, ParallelListener, ParallelDispatchResult,
};

#[derive(Clone, Eq, Hash, PartialEq)]
enum Event {
    Variant,
}

struct Listener {}

impl ParallelListener<Event> for Listener {
    fn on_event(&self, _event: &Event) -> Option<ParallelDispatchResult> {
        println!("I'm listening! :)");

        None
    }
}

fn main() {
    let listener = Listener {};
    let mut dispatcher = ParallelDispatcher::<Event>::new(4).expect("Could not construct threadpool");

    dispatcher.add_listener(Event::Variant, listener);
    dispatcher.dispatch_event(&Event::Variant);
}

```

# Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hey_listen = "0.4"
```

[ci-badge]: https://img.shields.io/azure-devops/build/lakeware/1942ff94-1b1e-4422-be98-1cd4696568d1/7/breaking-changes.svg?style=flat-square
[ci]: https://dev.azure.com/lakeware/hey_listen/_build?definitionId=7

[docs-badge]: https://img.shields.io/badge/docs-online-5023dd.svg?style=flat-square&colorB=32b6b7
[docs]: https://docs.rs/hey_listen

[rust version badge]: https://img.shields.io/badge/rust-1.34.1+-93450a.svg?style=flat-square&colorB=ff9a0d
[rust version link]: https://blog.rust-lang.org/2019/04/25/Rust-1.34.1.html

[crates.io link]: https://crates.io/crates/hey_listen
[crates.io version]: https://img.shields.io/crates/v/hey_listen.svg?style=flat-square&colorB=b73732
