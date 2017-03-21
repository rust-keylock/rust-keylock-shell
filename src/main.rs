extern crate rust_keylock;
extern crate log;
extern crate fern;
extern crate chrono;
extern crate rpassword;

mod logger;
mod shell;

#[allow(dead_code)] 
fn main() {
	logger::init_logging();
	let shell = shell::new();
	rust_keylock::execute(&shell);
}