use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[arg(long)]
    pub tcp_addr: String,

    #[arg(long)]
    pub tcp_port: String,

    #[arg(long)]
    pub udp_port: String, 

    #[arg(long)]
    pub filename: String, 
}