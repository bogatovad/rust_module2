use crate::errors::ErrorParsingCommand;
use crate::enums::{Command, CommandType};

pub fn parse_command(line: &String) -> Result<Command, ErrorParsingCommand>{
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
        udp_addr: upd_addr.trim_start_matches("udp://").to_string(),
        stocks: stocks,
    })
}