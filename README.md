# Elm Architecture Examples in Rust

This is a collection of examples from https://guide.elm-lang.org implementing
"The Elm Architecture", sometimes also called the Elm pattern. The examples are
implemented as simple console applications instead of being web apps. So,
instead of clicking on some buttons, you usually press keyboard keys, and
instead of showing HTML, there is just some output to console.

One difference to the pattern is the signature of the function `subscriptions`.
In Elm it takes a model and depending on it, it returns subscriptions. The
function is executed every time the model is updated. This makes it possible
to remove or cancel subscriptions previously created.

In Rust, the signature is:

```rust
fn subscriptions(Model) -> (Model, Sub<Msg>);
```

The function is called only once on initialization. The initial model is passed
and returned again, since on contrary to Elm we are not having a global state
but store everything in the model (e.g. Elm tracks open websockets in its
runtime and so allows magically to send values, even without having a channel
to the socket.)

Practically, it should be possible to call the function after each update of
the model and update current subscriptions, however, this also means that we
need some kind of tracking and identification of subscriptions already created.
Instead, we track a subscription with tokio's cancellation tokens (any other
kind of a cancellation token of a stream is also possible); see the `time`
example.

## How to run

```shell
cargo run --bin time
cargo run --bin random
cargo run --bin http
cargo run --bin websocket
```

## License

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT License ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this document by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
