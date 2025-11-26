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

const START_PRICE: f64 = 1000.0;
const MULTIPLAY_PRICE: f64 = 5000.0;
const START_PRICE_NOT_POPULAR_TICKERS: f64 = 100.0;
const TICKERS_FILENAME: &str = "tickers.txt";
const TIMEOUT_GENERATOR_SEC: u64 = 1;

fn generate_volume(ticker: &str) -> u32{
    match ticker {
        "AAPL" | "MSFT" | "TSLA" => START_PRICE as u32 + (rand::random::<f64>() * MULTIPLAY_PRICE) as u32,
        _ => START_PRICE_NOT_POPULAR_TICKERS as u32 + (rand::random::<f64>() * START_PRICE) as u32,
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
            price: rand::random::<f64>() * START_PRICE,
            volume: volume,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
        };
        stocks.push(serde_json::to_string(&stock_quote)?);
    }
    
    stocks.shuffle(&mut rand::thread_rng());
    Ok(stocks)
}

/// generate quote and send them to pipe.
pub fn generate_quote_daemon(tx: Sender<String>)-> Result<(), ErrorStockQuote>{
    loop{
        let data = generate_quote(TICKERS_FILENAME)?;
        for item in data{
            tx.send(item)?;
        }
        std::thread::sleep(std::time::Duration::from_secs(TIMEOUT_GENERATOR_SEC));
    }
}