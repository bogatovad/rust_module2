#[derive(Debug)]
pub enum CommandType{
    STREAM,
}

#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType,
    pub udp_addr: String,
    pub stocks: Vec<String>
}