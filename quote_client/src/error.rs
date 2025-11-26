use std::string::FromUtf8Error;
use std::sync::mpsc::SendError;
use std::net::AddrParseError;
use std::fmt;
use std::sync::mpsc::RecvError;
use std::time::SystemTimeError;

pub enum ConnectionResult {
    Exit,
    Lost,
    Ok
}

#[derive(Debug)]
pub enum ErrorRunningClient{
    ConnectionLost,
    ConnectFailed,
    SockAddrParseError,
    ErrorParseError,
    IoError
}

#[derive(Debug)]
pub enum ErrorReadUdp{
    ErrorUdp,
    ErrorFromUtf8Error,
    SendError,
    SockAddrError,
    RecvError,
    SystemTimeError
}

#[derive(Debug)]
pub enum ErrorHadleConnection{
    ErrorConnection,
    ErrorParseError
}

impl From<std::io::Error> for ErrorHadleConnection{
    fn from(error: std::io::Error) -> Self {
        ErrorHadleConnection::ErrorConnection
    }
}

impl From<std::io::Error> for ErrorReadUdp{
    fn from(error: std::io::Error) -> Self {
        ErrorReadUdp::ErrorUdp
    }
}

impl From<AddrParseError> for ErrorRunningClient{
    fn from(error: AddrParseError) -> Self {
        ErrorRunningClient::SockAddrParseError
    }
}

impl From<AddrParseError> for ErrorHadleConnection{
    fn from(error: AddrParseError) -> Self {
        ErrorHadleConnection::ErrorParseError
    }
}

impl From<ErrorHadleConnection> for ErrorRunningClient{
    fn from(error: ErrorHadleConnection) -> Self {
        ErrorRunningClient::ErrorParseError
    }
}

impl From<FromUtf8Error> for ErrorReadUdp{
    fn from(error: FromUtf8Error) -> Self {
        ErrorReadUdp::ErrorFromUtf8Error
    }
}

impl From<SendError<std::string::String>> for ErrorReadUdp{
    fn from(error: SendError<std::string::String>) -> Self {
        ErrorReadUdp::SendError
    }
}

impl From<std::io::Error> for ErrorRunningClient{
    fn from(error: std::io::Error) -> Self {
        ErrorRunningClient::IoError
    }
}

impl From<SendError<std::net::SocketAddr>> for ErrorReadUdp{
    fn from(error: SendError<std::net::SocketAddr>) -> Self {
        ErrorReadUdp::SockAddrError
    }
}

impl From<RecvError> for ErrorReadUdp{
    fn from(error: RecvError) -> Self {
        ErrorReadUdp::RecvError
    }
}

impl From<SystemTimeError> for ErrorReadUdp{
    fn from(error: SystemTimeError) -> Self {
        ErrorReadUdp::SystemTimeError
    }
}

impl fmt::Display for ErrorRunningClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorRunningClient::ConnectionLost => write!(f, "Error while tcp connecting"),
            ErrorRunningClient::ConnectFailed => write!(f, "Connect failed"),
            ErrorRunningClient::SockAddrParseError => write!(f, "Error while parsing tcp addr"),
            ErrorRunningClient::ErrorParseError => write!(f, "Error while parse"),
            ErrorRunningClient::IoError => write!(f, "IO Error")
        }
    }
}