use std::io::{self, Read};
use rand::seq::SliceRandom;
use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl StockQuote {
    pub fn to_string(&self) -> String {
        format!("{}|{}|{}|{}", self.ticker, self.price, self.volume, self.timestamp)
    }
    
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                volume: parts[2].parse().ok()?,
                timestamp: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.ticker.as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.price.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.volume.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.timestamp.to_string().as_bytes());
        bytes
    }
}

/// to use own ErrorParsingCommand
impl From<io::Error> for ErrorStockQuote {
    fn from(error: io::Error) -> Self {
        ErrorStockQuote::ErrorOpenFile
    }
}

#[derive(Debug)]
pub enum ErrorStockQuote {
    ErrorOpenFile
}

impl fmt::Display for ErrorStockQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorStockQuote::ErrorOpenFile => write!(f, "Stock quote file not found"),
        }
    }
}

impl Error for ErrorStockQuote {}

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

pub fn generate_quote(filepath: &str) -> Result<Vec<String>, ErrorStockQuote> {  
    let tickers = read_tickers_from_file(&filepath)?;
    let mut stocks: Vec<String> = Vec::new();

    for ticker in tickers{
        let volume = generate_volume(&ticker);
        let stock_quote = StockQuote {
            ticker: ticker.clone(),
            price: rand::random::<f64>() * 1000.0,
            volume: volume,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
        };
        stocks.push(serde_json::to_string(&stock_quote).unwrap());
    }
    
    stocks.shuffle(&mut rand::thread_rng());
    Ok(stocks)
}