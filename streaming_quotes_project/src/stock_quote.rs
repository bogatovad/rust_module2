use crate::errors::ErrorStockQuote;

use std::io::Read;
use rand::seq::SliceRandom;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crossbeam_channel::Sender;

#[derive(Serialize, Deserialize, Debug)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

fn generate_volume(ticker: &str) -> u32{
    match ticker {
        "AAPL" | "MSFT" | "TSLA" => 1000 + (rand::random::<f64>() * 5000.0) as u32,
        _ => 100 + (rand::random::<f64>() * 1000.0) as u32,
    }
}

fn read_tickers_from_file(filepath: &str)->Result<Vec<String>, ErrorStockQuote>{
    let mut content = String::new();
    let mut file_handelr = File::open(filepath)?;
    file_handelr.read_to_string(&mut content)?;
    Ok(content.split('\n').map(|value| value.to_string()).collect())
}

fn generate_quote(filepath: &str) -> Result<Vec<String>, ErrorStockQuote> {  
    let tickers = read_tickers_from_file(&filepath)?;
    let mut stocks: Vec<String> = Vec::new();

    for ticker in tickers{
        let volume = generate_volume(&ticker);
        let stock_quote = StockQuote {
            ticker: ticker.clone(),
            price: rand::random::<f64>() * 1000.0,
            volume: volume,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
        };
        stocks.push(serde_json::to_string(&stock_quote).unwrap());
    }
    
    stocks.shuffle(&mut rand::thread_rng());
    Ok(stocks)
}

/// generate quote and send them to pipe.
pub fn generate_quote_daemon(tx: Sender<String>)-> Result<(), ErrorStockQuote>{
    loop{
        let data = generate_quote("tickers.txt")?;
        for item in data{
            tx.send(item)?;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}