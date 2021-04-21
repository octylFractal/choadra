#![deny(warnings)]

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::exit;
use std::time::Instant;

use anyhow::Context;
use console::style;
use structopt::StructOpt;

use choadra::client::{ChoadraClient, Credentials};
use choadra::error::ChoadraError;
use choadra::protocol::datatype::position::Position;
use choadra::protocol::handshake::c2s::CURRENT_PROTOCOL_VERSION;
use choadra::protocol::play::c2s::{
    ChatMessage, ClientStatus, ClientStatusAction, KeepAlive, PlayerDigging, PlayerDiggingFace,
    PlayerDiggingStatus,
};
use choadra::protocol::play::s2c::S2CPlayPacket;
use choadra_executables::auth::authenticate_if_needed;
use choadra_executables::ezconsole::{new_style_e, style_e, TextComponent};

#[derive(StructOpt, Debug)]
#[structopt(name = "choadra-interact-spammer")]
struct ChoadraInteractSpammer {
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
    let args: ChoadraInteractSpammer = ChoadraInteractSpammer::from_args();
    if let Err(error) = main_for_result(args) {
        eprintln!("{}", style_e(format!("Error: {:?}", error)).red());
        exit(1);
    }
}

fn main_for_result(args: ChoadraInteractSpammer) -> anyhow::Result<()> {
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

    spam(socket_addr, username, credentials)?;

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

    if protocol != CURRENT_PROTOCOL_VERSION {
        return Err(anyhow::Error::new(ChoadraError::ServerError {
            msg: format!(
                "{} implements protocol version {}, not {} like we do.",
                socket_addr, protocol, CURRENT_PROTOCOL_VERSION
            ),
        }));
    }

    Ok(())
}

fn spam(
    socket_addr: SocketAddr,
    username: String,
    credentials: Option<Credentials>,
) -> anyhow::Result<()> {
    let stream = TcpStream::connect(socket_addr.clone())
        .with_context(|| format!("Failed to connect to {:?}", socket_addr))?;
    stream
        .set_read_timeout(None)
        .expect("set_read_timeout call failed");
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

    play_client.send_play_packet(ClientStatus {
        action: ClientStatusAction::Respawn,
    })?;
    play_client.send_play_packet(ChatMessage {
        message: "/clear".to_string(),
    })?;
    play_client.send_play_packet(ChatMessage {
        message: "//wand".to_string(),
    })?;
    play_client.send_play_packet(ChatMessage {
        message: "/tp -251 64 94".to_string(),
    })?;

    let mut last_packet_time = Instant::now();
    let mut position_flip = false;

    loop {
        let next = play_client.read_play_packet()?;
        match next {
            S2CPlayPacket::ChatMessage(cm) => {
                println!("Chat: [{:?}] <{}> {}", cm.position, cm.sender, cm.message);
            }
            S2CPlayPacket::KeepAlive(ka) => {
                println!("{}", style("Responding to KA request").green());
                play_client.send_play_packet(KeepAlive { id: ka.id })?;
            }
            S2CPlayPacket::Disconnect(d) => {
                println!("Disconnected: {}", d.reason);
                break;
            }
            _ => {}
        }
        if last_packet_time.elapsed().as_millis() >= 10 {
            position_flip = !position_flip;
            play_client.send_play_packet(PlayerDigging {
                status: PlayerDiggingStatus::StartedDigging,
                location: Position::try_new(-251, 62 + (position_flip as i16), 94)
                    .expect("It's a valid position"),
                face: PlayerDiggingFace::Top,
            })?;
            last_packet_time = Instant::now();
        }
    }

    Ok(())
}
