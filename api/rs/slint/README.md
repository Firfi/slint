# Slint

[![Crates.io](https://img.shields.io/crates/v/slint)](https://crates.io/crates/slint)
[![Docs.rs](https://docs.rs/slint/badge.svg)](https://docs.rs/slint)

# A Rust UI toolkit

[Slint](https://slint-ui.com/) is a UI toolkit that supports different programming languages.
Slint is the Rust API to interact with a Slint UI design from Rust.

The complete Rust documentation can be viewed online at https://slint-ui.com/docs/rust/slint/.

## Getting Started

The [crate documentation](https://slint-ui.com/docs/rust/slint/) shows how to use this crate.

### Hello World

The most basic "Hello world" application can be achieved with a few lines of code:

In your `Cargo.toml` add:

```toml
[dependencies]
slint = "0.3.3"
```

And in your `main.rs`:

```rust
slint::slint!{
    HelloWorld := Window {
        Text {
            text: "hello world";
            color: green;
        }
    }
}
fn main() {
    HelloWorld::new().run();
}
```

The [`slint` crate documentation](https://slint-ui.com/docs/rust/slint/)
contains more advanced examples and alternative ways to use this crate.

To quickly get started, you can use the [Template Repository](https://github.com/slint-ui/slint-rust-template) with
the code of a minimal application using Slint that can be used as a starting point to your program.

```bash
cargo install cargo-generate
cargo generate --git https://github.com/slint-ui/slint-rust-template
```

## More examples

You can quickly try out the [examples](/examples) by cloning this repo and running them with `cargo run`

```sh
# Runs the "printerdemo" example
cargo run --release --bin printerdemo
```

### Minimum Supported Rust Version

 This crate's minimum supported `rustc` version is 1.64.
