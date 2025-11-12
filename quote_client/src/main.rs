pub mod args;

use crate::args::Args;
use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, Instant};
use std::net::UdpSocket;
use std::sync::Arc;

enum ConnectionResult {
    Exit,
    Lost,
}

/// read data via UDP and send PING.
fn read_udp_data(udp_addr: String){
    loop{
        let mut buf = [0u8; 1024];
        let socket = UdpSocket::bind(&udp_addr).unwrap();
        let (size, src) = socket.recv_from(&mut buf).unwrap();
        let message = String::from_utf8(buf[..size].to_vec()).unwrap();
        println!("received: {}", message);
        socket.send_to(b"PING", &src).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
}


fn main() {
    let args = Args::parse();
    let tcp_addr = format!("{}:{}", args.tcp_addr, args.tcp_port);
    let udp_addr = format!("{}:{}", args.tcp_addr, args.udp_port);
    let addr: SocketAddr = tcp_addr.parse().unwrap();

    // read UDP data from server.
    thread::spawn(|| {read_udp_data(udp_addr)});

    loop {
        match connect(&addr) {
            Ok(stream) => {
                println!("Connected to server!");
                match handle_connection(stream) {
                    ConnectionResult::Exit => break,
                    ConnectionResult::Lost => {
                        println!("Connection lost. Reconnecting in 2s...");
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            Err(err) => {
                eprintln!("Connect failed: {}. Retrying in 2s...", err);
                thread::sleep(Duration::from_secs(2));
            }
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
                .with_time(Duration::from_secs(10))
                .with_interval(Duration::from_secs(5)),
        )?;
    }

    socket.connect(&addr.clone().into())?;
    let stream: TcpStream = socket.into();
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    Ok(stream)
}

fn handle_connection(stream: TcpStream) -> ConnectionResult {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let stdin = io::stdin();
    let mut line = String::new();

    if reader.read_line(&mut line).is_ok() {
        print!("{}", line);
    }

    loop {
        print!("vault> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();

        if stdin.read_line(&mut input).is_err() {
            return ConnectionResult::Lost;
        }

        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.eq_ignore_ascii_case("EXIT") {
            println!("Bye!");
            return ConnectionResult::Exit;
        }

        match send_command(&stream, &mut reader, trimmed) {
            Ok(response) => print!("{}", response),
            Err(e) => {
                println!("ERROR: connection lost ({})", e);
                return ConnectionResult::Lost;
            }
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
