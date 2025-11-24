pub mod args;
pub mod error;
use crate::args::Args;
use crate::error::{ErrorReadUdp, ErrorRunningClient, ConnectionResult, ErrorHadleConnection};
use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::Bytes;
use std::thread;
use std::time::Duration;
use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[macro_use]
extern crate log;
extern crate env_logger;

const KEEPALIVE_TIME: u64 = 10;
const KEEPALIVE_INVERVAL: u64 = 5;
const TIMEOUTE_READ_STREAM: u64 = 3;
const TIMEOUTE_PING_MESSAGE: u64 = 3;

/// read data via UDP and send PING.
fn read_udp_data(socket: Arc<UdpSocket>, tx: Sender<String>, tx_ping: Sender<SocketAddr>) -> Result<(), ErrorReadUdp>{
    loop{
        let mut buf = [0u8; 1024];
        let (size, src) = socket.recv_from(&mut buf)?;
        let message = String::from_utf8(buf[..size].to_vec())?;
        tx.send(message)?;
        tx_ping.send(src)?;
    }
}

/// send PING - timestamp.
fn send_ping(socket: Arc<UdpSocket>, rx: Receiver<SocketAddr>) -> Result<(), ErrorReadUdp>{
    let src = rx.recv()?;
    loop{
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        socket.send_to(timestamp.as_secs().to_string().as_bytes(), &src)?;
        std::thread::sleep(std::time::Duration::from_secs(TIMEOUTE_PING_MESSAGE));
    }
}

fn read_tickers_from_file(filename: &String) -> Result<String, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut tickers: String = String::new();

    for line in reader.lines() {
        let current_ticker = format!("{},", line?); 
        tickers.push_str(&current_ticker);
    }
    tickers.pop();

    Ok(tickers)
}

fn main() -> Result<(), ErrorRunningClient> {
    env_logger::init();
    let args = Args::parse();
    let tcp_addr = format!("{}:{}", args.tcp_addr, args.tcp_port);
    let udp_addr = format!("{}:{}", args.tcp_addr, args.udp_port);
    let filename = args.filename;
    let addr: SocketAddr = tcp_addr.parse()?;
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (tx_ping, rx_ping): (Sender<SocketAddr>, Receiver<SocketAddr>) = mpsc::channel();
    let tx_clone = tx.clone();
    let tx_ping_clone = tx_ping.clone();
    let clone_udp_addr = udp_addr.clone();

    let socket = UdpSocket::bind(&udp_addr)?;
    let sock_arc = Arc::new(socket);

    let sock_clone_read_udp = Arc::clone(&sock_arc);
    let sock_clone_send_ping = Arc::clone(&sock_arc);

    // read UDP data from server.
    thread::spawn(move || {
        let _ = read_udp_data(sock_clone_read_udp, tx_clone, tx_ping_clone);
    });

    // send PING.
    thread::spawn(move || {
        let _ = send_ping(sock_clone_send_ping, rx_ping);
    });

    //read tickers from the file.
    let tickers = read_tickers_from_file(&filename)?;

    match connect(&addr) {
        Ok(stream) => {
            info!("Connected to server!");
            match handle_connection(stream, clone_udp_addr, tickers)? {
                ConnectionResult::Exit => Ok(()),
                ConnectionResult::Lost => {
                    error!("Connection lost");
                    Err(ErrorRunningClient::ConnectionLost)
                },
                ConnectionResult::Ok => {
                    info!("Read UDP Strem here.");
                    for message in rx.iter(){
                        info!("{}", message);
                    }
                    Ok(())
                }
            }
        }
        Err(err) => {
            eprintln!("Connect failed: {}", err);
            Err(ErrorRunningClient::ConnectFailed)
        }
    }
}

fn connect(addr: &SocketAddr) -> io::Result<TcpStream> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_keepalive(true)?;

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        socket.set_tcp_keepalive(
            &socket2::TcpKeepalive::new()
                .with_time(Duration::from_secs(KEEPALIVE_TIME))
                .with_interval(Duration::from_secs(KEEPALIVE_INVERVAL)),
        )?;
    }

    socket.connect(&addr.clone().into())?;
    let stream: TcpStream = socket.into();
    stream.set_read_timeout(Some(Duration::from_secs(TIMEOUTE_READ_STREAM)))?;
    Ok(stream)
}

fn handle_connection(stream: TcpStream, udp_addr: String, tickers: String) -> Result<ConnectionResult, ErrorHadleConnection> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let command = format!("STREAM udp://{} {}", udp_addr, tickers);

    match send_command(&stream, &mut reader, &command) {
        Ok(response) => {
            Ok(ConnectionResult::Ok)
        },
        Err(e) => {
            error!("ERROR: connection lost ({})", e);
            return Ok(ConnectionResult::Lost);
        }
    }
}

fn send_command(
    mut stream: &TcpStream,
    reader: &mut BufReader<TcpStream>,
    command: &str,
) -> io::Result<String> {
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut buffer = String::new();
    let bytes = reader.read_line(&mut buffer)?;

    if bytes == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Server closed connection",
        ));
    }
    Ok(buffer)
}
