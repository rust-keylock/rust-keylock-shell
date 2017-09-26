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
        .chain(fern::log_file("output.log")?)
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
