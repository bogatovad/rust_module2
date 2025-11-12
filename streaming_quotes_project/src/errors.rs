use std::io;
use std::error::Error;
use std::fmt;
use std::time::SystemTimeError;

type SendErrorСrossbeam = crossbeam_channel::SendError<String>;

/// to use own ErrorParsingCommand.
impl From<io::Error> for ErrorParsingCommand {
    fn from(error: io::Error) -> Self {
        ErrorParsingCommand::ErrorHandleClient
    }
}

pub enum ErrorParsingCommand {
    MissingCommandType,
    MissingUdpAddr,
    MissingStocks, 
    ErrorHandleClient,
    InvalidCommandType
}

/// to use own ErrorParsingCommand
impl From<io::Error> for ErrorStockQuote {
    fn from(error: io::Error) -> Self {
        ErrorStockQuote::ErrorOpenFile
    }
}

#[derive(Debug)]
pub enum ErrorStockQuote {
    ErrorOpenFile,
    ErrorSend,
    ErrorParseDate
}

impl fmt::Display for ErrorStockQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorStockQuote::ErrorOpenFile => write!(f, "Error while opening file"),
            ErrorStockQuote::ErrorSend => write!(f, "Error while sending data"),
            ErrorStockQuote::ErrorParseDate => write!(f, "Error while parsing date")
        }
    }
}

impl Error for ErrorStockQuote {}

impl From<SendErrorСrossbeam> for ErrorStockQuote {
    fn from(error: SendErrorСrossbeam) -> Self {
        ErrorStockQuote::ErrorSend
    }
}

impl From<SystemTimeError> for ErrorStockQuote{
    fn from(error: SystemTimeError) -> Self {
        ErrorStockQuote::ErrorParseDate
    }
}