// Copyright 2017 astonbitecode
// This file is part of rust-keylock password manager.
//
// rust-keylock is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-keylock is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-keylock.  If not, see <http://www.gnu.org/licenses/>.
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
    rust_keylock::execute(Box::new(shell));
}
