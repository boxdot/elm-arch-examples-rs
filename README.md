# Elm Architecture Examples in Rust

This is a collection of examples from https://guide.elm-lang.org implementing
"The Elm Architecture", sometimes also called the Elm pattern. The examples are
implemented as simple console applications instead of being web apps. So,
instead of clicking on some buttons, you usually press keyboard keys, and
instead of showing HTML, there is just some output to console.

One difference to the pattern is the signature of the function `subscriptions`.
In Elm it takes a model and depending on it, it returns subscriptions. This
makes it possible to remove or cancel subscriptions previously created. This is
not implemented in Rust; the signature is:

```rust
fn subscriptions: () -> Sub<Msg>;
```

If you find interesting or challenging to implement this, feel free to create a pull request.

## How to run

```shell
crate run --bin time
crate run --bin random
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
