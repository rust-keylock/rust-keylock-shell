![Build Status](https://travis-ci.org/rust-keylock/rust-keylock-shell.svg?branch=master)
[![crates.io](https://img.shields.io/crates/v/rust_keylock_shell.svg)](https://crates.io/crates/rust_keylock_shell)

___rust-keylock-shell___ is the [Editor](https://rust-keylock.github.io/rust-keylock-lib/rust_keylock/trait.Editor.html) that manages the [rust-keylock-lib](https://github.com/rust-keylock/rust-keylock-lib) via the shell.

## General

___rust-keylock___ is a password manager and its goals are to be:

* Secure
* Simple to use
* Portable
* Extensible

The core logic is written in [Rust](https://www.rust-lang.org), but the presentation/User interaction parts are in different languages.

## Features

### Security

 * The data is locked with a user-defined master password, using _bcrypt_ password hashing
 * Encryption using _AES_ with _CTR_ mode
 * Data integrity checks with SHA3 (Keccak)
 * Encrypted bytes blending
 * Passwords are kept encrypted in memory
 * Encryption keys on runtime are stored in safe, non-swappable memory
 * Encryption keys change upon saving, even if the user master password remains the same. This results to different encrypted products, even if the data that is being encrypted is the same.
 
### Application Portability

 * [Shell implementation](https://github.com/rust-keylock/rust-keylock-shell) running on Linux and Windows
 * [JavaFX implementation](https://github.com/rust-keylock/rust-keylock-ui) running on Linux and Windows
 * [Android implementation](https://github.com/rust-keylock/rust-keylock-android) soon to be published in [F-Droid](https://gitlab.com/fdroid/fdroiddata/merge_requests/2141)

Thanks to [xargo](https://github.com/japaric/xargo), [cross](https://github.com/japaric/cross) and [JNA](https://github.com/java-native-access/jna)!
 
### Import/export mechanism

 * Export/import encrypted passwords to/from the filesystem

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
 cargo build --release
 ```
 
* Run:

 ```shell
 ./target/release/rust_keylock`
 ```