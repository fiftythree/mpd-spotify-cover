use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::str::FromStr;
use std::time::Duration;
use std::error::Error;
use log::*;

use crate::config;

const MPD_COMMAND_CURRENT_SONG: &str = "currentsong";

pub fn get_playing_song_props(config: &config::Config)
  -> Result<HashMap<String, String>, Box<dyn Error>> {

    let mut stream = connect(config)?;
    let mut reader = BufReader::new(stream.try_clone()?);

    {
        let mut hello_line = String::new();
        reader.read_line(&mut hello_line)?;

        if !hello_line.starts_with("OK MPD") {
            return Err(
                format!("received an arbitrary welcome line: {}", hello_line))?;
        }

        debug!("Connected to MPD version {}", hello_line
            .get("OK MPD ".len()..hello_line.len())
            .expect("no MPD version").trim());
    }

    writeln!(stream, "{}", MPD_COMMAND_CURRENT_SONG)?;

    // read MPD response as lines
    let lines = read_all(&mut reader)?;

    // convert key-value pairs to a hashmap
    Ok(lines
        .iter()
        .map(|i| {
            let opt: Option<(String, String)> = try {
                let index = i.find(": ")?;

                (
                    String::from(i.get(0..index)?),
                    String::from(i.get(index + ": ".len()..i.len())?)
                )
            };

            opt.expect("bad data")
        })
        .into_iter()
        .collect::<HashMap<String, String>>())
}

fn read_all(reader: &mut BufReader<TcpStream>)
  -> Result<Vec<String>, Box<dyn Error>> {

    let mut lines: Vec<String> = Vec::new();

    loop {
        let mut content = String::new();
        reader.read_line(&mut content)?;

        content = String::from(content.trim());

        if content.eq("OK") {
            break
        }

        lines.push(content);
    }

    Ok(lines)
}

fn connect(config: &config::Config)
  -> Result<TcpStream, Box<dyn Error>> {

    let ip_address = IpAddr::from_str(config.mpd.address.as_str())?;

    Ok(
        TcpStream::connect_timeout(
            &SocketAddr::from((ip_address, config.mpd.port)),
            Duration::from_secs(30)
        )?
    )
}