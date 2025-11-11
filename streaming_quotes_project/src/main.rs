pub mod stock_quote;

use crate::stock_quote::StockQuote;

use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use std::io::Write;
use std::io::BufRead;
use std::io::BufReader;

use std::io;

//STREAM udp://127.0.0.1:34254 AAPL,TSLA.
use std::net::UdpSocket;
use std::time::Duration;
use crate::stock_quote::generate_quote;

use std::sync::mpsc;

use crossbeam_channel::{unbounded, Sender, Receiver};

// todo: 1) написать генератор котироков
// 2) оформить в виде отдельного приложения первый модуль
// 3) создать отдельное приложение клиент для клиента
// 4) запустить все вместе

pub struct StockSender {
    socket: UdpSocket
}

impl StockSender {
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind("127.0.0.1:0")?;
        Ok(Self { socket })
    }

    // Метод отправки сообщений в сокет
    pub fn send_to(
        &self,
        stock: &String,
        target_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.socket.send_to(stock.as_bytes(), "127.0.0.1:8080")?;
        Ok(())
    }

    pub fn read_ping(&self){

        loop{
            let mut buf = [0u8; 1024];
            //let _ = self.socket.recv(&mut buf).unwrap();

            let (size, src) = self.socket.recv_from(&mut buf).unwrap();
            let message = String::from_utf8(buf[..size].to_vec()).unwrap();
            println!("FROM CLIENT: {}", message);
        }
    }

    // Метод для запуска цикла постоянной отправки метрик
    pub fn start_broadcasting(
        self,
        target_addr: &String,
        interval_ms: u64,
        rx: Receiver<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop{
            let data = rx.recv()?;
            match self.send_to(&data, &target_addr) {
                Ok(()) => {
                    println!(
                        "Data sent. {}", data
                    );
                    std::thread::sleep(std::time::Duration::from_millis(600));
                }
                Err(e) => {
                    eprintln!("Ошибка отправки: {}", e);
                }
            }
            thread::sleep(Duration::from_millis(interval_ms));
        }
    }
} 


#[derive(Debug)]
enum CommandType{
    STREAM
}


#[derive(Debug)]
struct Command {
    command_type: CommandType,
    udp_addr: String,
    stocks: Vec<String>
}

/// to use own ErrorParsingCommand
impl From<io::Error> for ErrorParsingCommand {
    fn from(error: io::Error) -> Self {
        ErrorParsingCommand::ErrorHandleClient
    }
}

enum ErrorParsingCommand {
    MissingCommandType,
    MissingUdpAddr,
    MissingStocks, 
    ErrorHandleClient,
   InvalidCommandType
}

fn parse_command(line: &String) -> Result<Command, ErrorParsingCommand>{
    let mut parts= line.split(" ");

    let command_type = parts.next()
        .ok_or(ErrorParsingCommand::MissingCommandType)
        .and_then(|cmd| match cmd {
            "STREAM" => Ok(CommandType::STREAM),
            _ => Err(ErrorParsingCommand::InvalidCommandType),
        });
    let upd_addr = String::from(parts.next().ok_or(ErrorParsingCommand::MissingUdpAddr)?);
    let stocks: Vec<String> = parts.next().ok_or(ErrorParsingCommand::MissingStocks)?.trim().split(',')
                .map(|value|{value.to_string()}).collect();

    Ok(Command{
        command_type: command_type?,
        udp_addr: upd_addr,
        stocks: stocks
    })
}

fn create_upd_connection(command: &Command, rx: Receiver<String>){
    let sender = StockSender::new(&command.udp_addr).unwrap();
    let _ = sender.start_broadcasting(&command.udp_addr, 100, rx);
}

/// method to process a new client.
fn handle_client(stream: TcpStream, rx: Receiver<String>) -> Result<(), ErrorParsingCommand>{
    println!("i am here");

    let mut writer = stream.try_clone().expect("failed to clone stream");
    let mut reader = BufReader::new(stream);

    let _ = writer.write_all(b"Welcome to my tcp server!\n");
    let _ = writer.flush();

    loop{
        println!("start");
        let mut line = String::new();
        reader.read_line(&mut line)?;

        println!("{:?}", line);
        let command: Command = parse_command(&line)?;

        println!("{:?}", command);
        let rx_clone = rx.clone();

        match command.command_type {
            CommandType::STREAM => {
                std::thread::spawn(move || {
                    create_upd_connection(&command, rx_clone)
                });
            }
        }
    }

}


fn read_udp_data(){
    loop{
        println!("hello");
        let socket = UdpSocket::bind("127.0.0.1:8080").unwrap();
        let mut buf = [0u8; 1024];
        println!("hello2");
        let (size, src) = socket.recv_from(&mut buf).unwrap();
        let message = String::from_utf8(buf[..size].to_vec()).unwrap();
        println!("Received: {}", message);
        std::thread::sleep(std::time::Duration::from_millis(800));
        let _ = socket.send_to("PING".as_bytes(), "127.0.0.1:8080");
    }
}

fn generate_qute_daemon(tx: Sender<String>)-> Result<(), Box<dyn std::error::Error>>{
    let mut couner = 1;
    loop{
        let data = generate_quote("tickers.txt")?;
        for item in data{
            tx.send(item).unwrap();
        }
        println!("SENDED ALL DATA {} times", couner);
        couner += 1;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server listening on port 7878");
    let (tx, rx) = unbounded::<String>();

    thread::spawn(move || {
        generate_qute_daemon(tx);
    });

    thread::spawn(read_udp_data);
    println!("go");

    for stream in listener.incoming() {
        println!("waiting");
        let rx_clone = rx.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream, rx_clone);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}
