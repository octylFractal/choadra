#![deny(warnings)]

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::exit;

use anyhow::Context;
use console::Style;
use structopt::StructOpt;

use choadra::client::{ChoadraClient, Credentials};
use choadra::error::ChoadraError;
use choadra::protocol::handshake::c2s::CURRENT_PROTOCOL_VERSION;

use choadra::protocol::play::c2s::KeepAlive;
use choadra::protocol::play::s2c::S2CPlayPacket;
use choadra_executables::auth::authenticate_if_needed;
use choadra_executables::ezconsole::{new_style_e, style_e, TextComponent};

#[derive(StructOpt, Debug)]
#[structopt(name = "choadra-dumper")]
struct ChoadraDumper {
    /// The host to connect to.
    host: String,
    /// If offline mode, the username to give to the server.
    /// If online mode, the username for your Mojang account, in-game username will be derived.
    username: String,
    /// Enable offline mode. If set to false, will be online mode instead.
    #[structopt(long)]
    offline: bool,
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
    let (username, credentials) = authenticate_if_needed(args.offline, args.username.clone())
        .context("Failed to authenticate")?;

    let socket_addr = (&*args.host, args.port)
        .to_socket_addrs()?
        .next()
        .with_context(|| format!("Unable to resolve {}", args.host))?;

    eprintln!(
        "{}",
        style_e(format!("Connected to {}!", socket_addr)).green()
    );

    check_status(socket_addr)?;

    interactive(socket_addr, username, credentials)?;

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

    eprintln!(
        "{}",
        TextComponent::of_style(new_style_e().bright().blue()).mutate_children(|c| {
            c.extend(vec![
                TextComponent::of_styled(socket_addr, new_style_e().cyan()),
                TextComponent::of(" describes itself as a "),
                TextComponent::of_styled(response.version.name, new_style_e().cyan()),
                TextComponent::of(" server, implementing protocol "),
                TextComponent::of_styled(response.version.protocol, new_style_e().cyan()),
                TextComponent::of(". Players: "),
                TextComponent::of_styled(response.players.online, new_style_e().red()),
                TextComponent::of("/"),
                TextComponent::of_styled(response.players.max, new_style_e().green()),
                TextComponent::of(". MOTD: "),
                TextComponent::of_styled(response.description.text, new_style_e().cyan()),
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

    eprintln!(
        "{}",
        TextComponent::of_style(new_style_e().bright().blue()).mutate_children(|c| {
            c.extend(vec![
                TextComponent::of("Ping time to "),
                TextComponent::of_styled(socket_addr, new_style_e().cyan()),
                TextComponent::of(" is "),
                TextComponent::of_styled(ping.as_millis(), new_style_e().cyan()),
                TextComponent::of_styled("ms", new_style_e().cyan()),
            ]);
        }),
    );

    Ok(())
}

fn interactive(
    socket_addr: SocketAddr,
    username: String,
    credentials: Option<Credentials>,
) -> anyhow::Result<()> {
    let stream = TcpStream::connect(socket_addr.clone())
        .with_context(|| format!("Failed to connect to {:?}", socket_addr))?;
    let mut play_client = ChoadraClient::new(stream)
        .request_login()
        .context("Failed to request login state from server")?
        .login(username.clone(), credentials)
        .context("Failed to login to server")?;

    eprintln!(
        "{}",
        TextComponent::of_style(new_style_e().bright().blue()).mutate_children(|c| {
            c.extend(vec![
                TextComponent::of("Logged in to "),
                TextComponent::of_styled(socket_addr, new_style_e().cyan()),
                TextComponent::of(" as "),
                TextComponent::of_styled(&play_client.state().username, new_style_e().cyan()),
                TextComponent::of(" with UUID: "),
                TextComponent::of_styled(&play_client.state().uuid, new_style_e().cyan()),
            ]);
        }),
    );

    loop {
        let next = play_client.read_play_packet()?;
        match &next {
            S2CPlayPacket::KeepAlive(ka) => {
                play_client.send_play_packet(KeepAlive { id: ka.id })?;
            }
            _ => {}
        }
        println!(
            "{}",
            TextComponent::of_style(Style::new().dim().green()).mutate_children(|c| {
                c.extend(vec![
                    TextComponent::of("Received play packet: "),
                    TextComponent::of_styled(format!("{:#X?}", next), Style::new().cyan()),
                ]);
            }),
        );
    }
}
