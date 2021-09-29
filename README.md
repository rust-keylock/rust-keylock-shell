![Build Status](https://app.travis-ci.com/rust-keylock/rust-keylock-shell.svg?branch=master)
[![crates.io](https://img.shields.io/crates/v/rust_keylock_shell.svg)](https://crates.io/crates/rust_keylock_shell)

___rust-keylock___ is a password manager and its goals are to be:

* Secure
* Simple to use
* Portable
* Extensible

___rust-keylock-shell___ provides command-line access to [rust-keylock-lib](https://github.com/rust-keylock/rust-keylock-lib).

# Warning

The project has not yet received any formal / official security reviews. Use it at your own risk.

# Install the shell Editor

Provided that Rust and Cargo are [installed](https://rustup.rs/), simply issue

```shell
cargo install rust_keylock_shell
```

Run the application:

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
 cargo build
 ```
 
* Run:

 ```shell
 ./target/release/rust_keylock`
 ```
 
# More info

* [FAQ](https://rust-keylock.github.io/faq/rkl/) 
* [Wiki](https://rust-keylock.github.io/wiki/)
