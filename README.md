![Build Status](https://travis-ci.org/rust-keylock/rust-keylock-shell.svg?branch=master)

___rust-keylock-shell___ is a shell handler of the [rust-keylock-lib](https://github.com/rust-keylock/rust-keylock-lib)

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