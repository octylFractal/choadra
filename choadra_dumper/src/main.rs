#![deny(warnings)]

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::exit;

use anyhow::Context;
use structopt::StructOpt;

use crate::ezconsole::{style_e, TextComponent};
use choadra::client::ChoadraClient;
use choadra::error::ChoadraError;
use choadra::protocol::handshake::c2s::CURRENT_PROTOCOL_VERSION;
use console::Style;

mod ezconsole;

#[derive(StructOpt, Debug)]
#[structopt(name = "choadra-dumper")]
struct ChoadraDumper {
    /// The host to connect to.
    host: String,
    // The username to connect with.
    username: String,
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

    eprintln!(
        "{}",
        style_e(format!("Connected to {}!", socket_addr)).green()
    );

    check_status(socket_addr)?;

    interactive(socket_addr, args.username)?;

    Ok(())
}

fn check_status(socket_addr: SocketAddr) -> anyhow::Result<()> {
    let stream = TcpStream::connect(socket_addr.clone())
        .with_context(|| format!("Failed to connect to {:?}", socket_addr))?;
    let mut status_client = ChoadraClient::new(stream)
        .request_status()
        .context("Failed to request status state from server")?;

    let response = status_client
        .status()
        .context("Failed to get status of server")?;
    let protocol = response.version.protocol;

    println!(
        "{}",
        TextComponent::of_style(Style::new().bright().blue()).mutate_children(|c| {
            c.extend(vec![
                TextComponent::of_styled(socket_addr, Style::new().cyan()),
                TextComponent::of(" describes itself as a "),
                TextComponent::of_styled(response.version.name, Style::new().cyan()),
                TextComponent::of(" server, implementing protocol "),
                TextComponent::of_styled(response.version.protocol, Style::new().cyan()),
                TextComponent::of(". Players: "),
                TextComponent::of_styled(response.players.online, Style::new().red()),
                TextComponent::of("/"),
                TextComponent::of_styled(response.players.max, Style::new().green()),
                TextComponent::of(". MOTD: "),
                TextComponent::of_styled(response.description.text, Style::new().cyan()),
            ]);
        }),
    );

    if protocol != CURRENT_PROTOCOL_VERSION {
        return Err(anyhow::Error::new(ChoadraError::ServerError {
            msg: format!(
                "{} implements protocol version {}, not {} like we do.",
                socket_addr, protocol, CURRENT_PROTOCOL_VERSION
            ),
        }));
    }

    let ping = status_client.ping().context("Failed to ping server")?;

    println!(
        "{}",
        TextComponent::of_style(Style::new().bright().blue()).mutate_children(|c| {
            c.extend(vec![
                TextComponent::of("Ping time to "),
                TextComponent::of_styled(socket_addr, Style::new().cyan()),
                TextComponent::of(" is "),
                TextComponent::of_styled(ping.as_millis(), Style::new().cyan()),
                TextComponent::of_styled("ms", Style::new().cyan()),
            ]);
        }),
    );

    Ok(())
}

fn interactive(socket_addr: SocketAddr, username: String) -> anyhow::Result<()> {
    let stream = TcpStream::connect(socket_addr.clone())
        .with_context(|| format!("Failed to connect to {:?}", socket_addr))?;
    let play_client = ChoadraClient::new(stream)
        .request_login()
        .context("Failed to request login state from server")?
        .login(username)
        .context("Failed to login to server")?;

    drop(play_client);

    loop {
        unreachable!();
    }
}
