use crate::stock_quote::StockQuote;

use std::net::UdpSocket;
use std::time::Duration;
use std::sync::Arc;
use crossbeam_channel::{Receiver, Sender};
use std::thread;

pub struct StockSender {
    socket: UdpSocket
}

const TIMEOUT_READ_PING_SEC: u64 = 5;

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
    pub fn run_read_ping(&self, tx_ping: Sender<String>) -> Result<(), Box<dyn std::error::Error>>{
        let sock = Arc::new(self.socket.try_clone()?);

        // if we don't ping-message during 2 sec then abort UDP stream.
        let _ = sock.set_read_timeout(Some(std::time::Duration::from_secs(TIMEOUT_READ_PING_SEC)));
        std::thread::spawn(move || {
            loop {
                // read ping message in loop.
                let mut buf = [0u8; 1024];
                let size = sock.recv(&mut buf);
                
                match size{
                    Ok(size) => {
                        info!("ping-mesage has been sent {} bytes", size);
                    },
                    Err(error) => {
                        error!("Ping timeout error with message{}", error);
                        tx_ping.send("CLOSE".to_string());
                        break
                    }
                }
            }
        });
        Ok(())
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
            let stock_quote: StockQuote = serde_json::from_str(&data)?;

            // implementing filter tickers.
            if !tickers.contains(&stock_quote.ticker){
                continue;
            }

            // send data via UDP to client.
            match self.send_to(&data, &target_addr) {
                Ok(()) => {
                    info!("data sent {}", data);
                    std::thread::sleep(std::time::Duration::from_millis(600));
                }
                Err(e) => {
                    info!("error while sending data: {}", e);
                }
            }

            // check error-ping message
            if let Ok(message) = rx_ping.try_recv() {
                if message == "CLOSE" {
                    error!("Stopping broadcast, closing socket...");
                    break ;
                }
            }
            thread::sleep(Duration::from_millis(interval_ms));
        }

        // return Ok if we break the circle.
        Ok(())
    }
} 