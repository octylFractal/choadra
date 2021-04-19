#![deny(warnings)]

use std::net::{TcpStream, ToSocketAddrs};
use std::process::exit;

use anyhow::Context;
use structopt::StructOpt;

use crate::ezconsole::style_e;
use choadra::client::ChoadraClient;

mod ezconsole;

#[derive(StructOpt, Debug)]
#[structopt(name = "choadra-dumper")]
struct ChoadraDumper {
    /// The host to connect to.
    host: String,
    /// The port to connect to.
    #[structopt(short, long, default_value = "25565")]
    port: u16,
}

fn main() {
    let args: ChoadraDumper = ChoadraDumper::from_args();
    if let Err(error) = main_for_result(args) {
        eprintln!("{}", style_e(format!("Error: {:?}", error)).red());
        exit(1);
    }
}

fn main_for_result(args: ChoadraDumper) -> anyhow::Result<()> {
    let socket_addr = (&*args.host, args.port)
        .to_socket_addrs()?
        .next()
        .with_context(|| format!("Unable to resolve {}", args.host))?;
    let stream = TcpStream::connect(socket_addr.clone())
        .with_context(|| format!("Failed to connect to {:?}", socket_addr))?;

    eprintln!(
        "{}",
        style_e(format!("Connected to {}!", socket_addr)).green()
    );

    let mut client = ChoadraClient::new(stream)
        .request_status()
        .context("Failed to request status packet from server")?;

    let ping = client.ping().context("Failed to ping server")?;

    println!("Ping time to {} is {}ms", socket_addr, ping.as_millis());

    Ok(())
}
