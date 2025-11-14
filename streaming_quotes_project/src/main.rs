pub mod stock_quote;
pub mod errors;
pub mod enums;
pub mod parse;
pub mod stock_sender;

use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use crossbeam_channel::{unbounded, Receiver};

use std::io::Write;
use std::io::BufRead;
use std::io::BufReader;

use crate::stock_quote::generate_quote_daemon;
use crate::errors::ErrorParsingCommand;
use crate::enums::{Command, CommandType};
use crate::parse::parse_command;
use crate::stock_sender::StockSender;


/// method to process a new client.
fn handle_client(stream: TcpStream, rx: Receiver<String>) -> Result<(), ErrorParsingCommand>{
    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    let _ = writer.write_all(b"Welcome to my tcp server!\n");
    let _ = writer.flush();

    // bind to local addr and change any free port.
    let upd_addr_local = "127.0.0.1:0";

    // read command from client.
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let command: Command = parse_command(&line)?;
    let rx_clone = rx.clone();

    match command.command_type {
        CommandType::STREAM => {
            std::thread::spawn(move || {
                let sender = StockSender::new(&upd_addr_local).unwrap();
                let _ = sender.start_broadcasting(&command.udp_addr, 100, rx_clone, &command.stocks);
            });
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tcp_addr_local = "127.0.0.1:7878";
    let listener = TcpListener::bind(tcp_addr_local)?;
    let (tx, rx) = unbounded::<String>();
    let tx_clone = tx.clone();

    thread::spawn(move ||{
        // run gerarator quotes.
        let _ = generate_quote_daemon(tx_clone);
    });

    for stream in listener.incoming() {
        let rx_clone = rx.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    // process TCP client.
                    let _ = handle_client(stream, rx_clone);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
