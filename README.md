![Build Status](https://travis-ci.org/rust-keylock/rust-keylock-shell.svg?branch=master)
[![crates.io](https://img.shields.io/crates/v/rust_keylock_shell.svg)](https://crates.io/crates/rust_keylock_shell)

___rust-keylock-shell___ is a shell handler of the [rust-keylock-lib](https://github.com/rust-keylock/rust-keylock-lib)

# Install

Provided that Rust and Cargo are [installed](https://rustup.rs/), simply issue

```shell
cargo install rust_keylock_shell
```

Run the application issuing:

`$ rust-keylock`


# Build

* Install [Rust](https://rustup.rs/)

* Clone the code:
 ```shell
 mkdir tmp
 cd tmp
 git clone https://github.com/rust-keylock/rust-keylock-shell.git
 
 ```

* Build:

 ```shell
 cd rust-keylock-shell
 cargo build --release
 ```
 
* Run:

 ```shell
 ./target/release/rust_keylock`
 ```