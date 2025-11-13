pub mod args;

use crate::args::Args;
use clap::Parser;
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{self, BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::Duration;
use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver, Sender};

enum ConnectionResult {
    Exit,
    Lost,
    Ok
}

/// read data via UDP and send PING.
fn read_udp_data(udp_addr: String, tx: Sender<String>){
    loop{
        let mut buf = [0u8; 1024];
        let socket = UdpSocket::bind(&udp_addr).unwrap();
        let (size, src) = socket.recv_from(&mut buf).unwrap();
        let message = String::from_utf8(buf[..size].to_vec()).unwrap();
        tx.send(message).expect("Error while send UDP message to main thread to print.");
        socket.send_to(b"PING", &src).unwrap();
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

fn main()  {
    let args = Args::parse();
    let tcp_addr = format!("{}:{}", args.tcp_addr, args.tcp_port);
    let udp_addr = format!("{}:{}", args.tcp_addr, args.udp_port);
    let filename = args.filename;
    let addr: SocketAddr = tcp_addr.parse().unwrap();
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let tx_clone = tx.clone();
    let clone_udp_addr = udp_addr.clone();

    // read UDP data from server.
    let _ = thread::spawn(move || {
        read_udp_data(udp_addr, tx_clone);
    });

    //read tickers from the file.
    let tickers = read_tickers_from_file(&filename).expect("Error read file tickers.");

    match connect(&addr) {
        Ok(stream) => {
            println!("Connected to server!");
            match handle_connection(stream, clone_udp_addr, tickers) {
                ConnectionResult::Exit => return,
                ConnectionResult::Lost => {
                    println!("Connection lost");
                },
                ConnectionResult::Ok => {
                    println!("Read UDP Strem here.");
                    for message in rx.iter(){
                        println!("{}", message);
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Connect failed: {}", err);
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

fn handle_connection(stream: TcpStream, udp_addr: String, tickers: String) -> ConnectionResult {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let command = format!("STREAM udp://{} {}", udp_addr, tickers);

    match send_command(&stream, &mut reader, &command) {
        Ok(response) => {
            ConnectionResult::Ok
        },
        Err(e) => {
            println!("ERROR: connection lost ({})", e);
            return ConnectionResult::Lost;
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
