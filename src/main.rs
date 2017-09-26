extern crate rust_keylock;
extern crate log;
extern crate fern;
extern crate chrono;
extern crate rpassword;

mod logger;
mod shell;

#[allow(dead_code)]
fn main() {
    let res = logger::init_logging();
    if res.is_err() {
    	println!("Could not initialize logger! Reason: {}", res.err().unwrap())
    }
    let shell = shell::new();
    rust_keylock::execute(&shell);
}
