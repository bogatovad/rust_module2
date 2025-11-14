use crate::stock_quote::StockQuote;

use std::net::UdpSocket;
use std::time::Duration;
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};
use std::thread;

pub struct StockSender {
    socket: UdpSocket
}

impl StockSender {
    /// create new udp socket.
    pub fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr)?;
        Ok(Self { socket })
    }

    /// send to socket message.
    pub fn send_to(
        &self,
        stock: &String,
        target_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.socket.send_to(stock.as_bytes(), target_addr)?;
        Ok(())
    }
    
    /// method to read ping data from client.
    pub fn run_read_ping(&self, tx_ping: Sender<String>){
        let sock = Arc::new(self.socket.try_clone().unwrap());

        // if we don't ping-message during 2 sec then abort UDP stream.
        let _ = sock.set_read_timeout(Some(std::time::Duration::from_secs(5)));
        std::thread::spawn(move || {
            let result = std::panic::catch_unwind(|| {
                loop {

                    // read message in loop.
                    let mut buf = [0u8; 1024];
                    let (size, _src) = sock.recv_from(&mut buf).expect("Timeout while getting ping-message");
                    let message = String::from_utf8(buf[..size].to_vec()).unwrap();
                    println!("ping-mesage from client: {}", message);
                }
            });
            match result {
                Ok(()) => println!("it's ok"),
                Err(panic) => {

                    // send CLOSE message to close UDP stream if we catch panic.
                    let _ = tx_ping.send("CLOSE".to_string());
                    return;
                },
            }
        });
    }

    /// method to broadcast data to client via UDP protocol.
    pub fn start_broadcasting(
        self,
        target_addr: &String,
        interval_ms: u64,
        rx: Receiver<String>,
        tickers: &Vec<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crossbeam_channel::unbounded;
        let (tx_ping, rx_ping) = unbounded::<String>();
        let tx_clone = tx_ping.clone();
        
        // run throw to read ping-message from client.
        self.run_read_ping(tx_clone);

        loop{
            // read data from generator via pipe.
            let data = rx.recv()?;
            let stock_quote: StockQuote = serde_json::from_str(&data).unwrap();

            // implementing filter tickers.
            if !tickers.contains(&stock_quote.ticker){
                continue;
            }

            // send data via UDP to client.
            match self.send_to(&data, &target_addr) {
                Ok(()) => {
                    println!("data sent {}", data);
                    std::thread::sleep(std::time::Duration::from_millis(600));
                }
                Err(e) => {
                    eprintln!("error while sending data: {}", e);
                }
            }

            //check error-ping message
            if let Ok(message) = rx_ping.try_recv() {
                if message == "CLOSE" {
                    println!("Stopping broadcast, closing socket...");
                    break ;
                }
            }
            thread::sleep(Duration::from_millis(interval_ms));
        }

        // return Ok if we break the circle.
        Ok(())
    }
} 