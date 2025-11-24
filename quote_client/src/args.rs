use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    // BOX<str> - лайфхак Если тебя нужна строчка которую ты не будешь никогда мутировать, сохраняет тебе целых 1 usize байтов)
    #[arg(long)]
    pub tcp_addr: String, 

    #[arg(long)]
    pub tcp_port: String,

    #[arg(long)]
    pub udp_port: String, 

    #[arg(long)]
    pub filename: String, 
}