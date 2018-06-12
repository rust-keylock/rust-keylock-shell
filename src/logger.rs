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
use log;
use fern;
use chrono;
use std::fmt;
use std::error::Error;
use std::io;

pub fn init_logging() -> Result<(), ShellLoggerError> {
    fern::Dispatch::new().format(|out, message, record| {
            out.finish(format_args!("{}[{}][{}] {}",
                                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                                    record.target(),
                                    record.level(),
                                    message))
        })
        .level(log::LogLevelFilter::Debug)
        .chain(fern::log_file("rust-keylock.log")?)
        .apply()?;

	Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ShellLoggerError {
    error_message: String
}

impl fmt::Display for ShellLoggerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	write!(f, "{}", self.error_message)
	}
}

impl Error for ShellLoggerError {
    fn description(&self) -> &str {
    	"ShellLoggerError"
    }
}

impl From<io::Error> for ShellLoggerError {
    fn from(err: io::Error) -> ShellLoggerError {
    	ShellLoggerError {error_message: format!("{:?}", err)}
    }
}

impl From<log::SetLoggerError> for ShellLoggerError {
    fn from(err: log::SetLoggerError) -> ShellLoggerError {
    	ShellLoggerError {error_message: format!("{:?}", err)}
    }
}
