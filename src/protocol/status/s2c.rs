use binread::io::{Read, Seek, SeekFrom};
use binread::{derive_binread, BinRead, BinResult, ReadOptions};
use serde::Deserialize;

use crate::protocol::datatype::aliases::{Int, Long};
use crate::protocol::datatype::string::parse_string;

#[derive_binread]
#[br(import(id: Int))]
#[derive(Debug)]
pub enum S2CStatusPacket {
    #[br(pre_assert(id == 0))]
    Response(Response),
    #[br(pre_assert(id == 1))]
    Pong(Pong),
}

#[derive_binread]
#[derive(Debug)]
pub struct Pong(pub Long);

#[derive(Debug, Deserialize)]
pub struct Response {
    pub version: VersionInfo,
    pub players: PlayerInfo,
    pub description: DescriptionInfo,
    pub favicon: Option<String>,
}

impl BinRead for Response {
    type Args = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        options: &ReadOptions,
        _: Self::Args,
    ) -> BinResult<Self> {
        let string = parse_string(reader, options, (None,))?;
        serde_json::from_str(&string).or_else(|e| {
            let pos = reader.seek(SeekFrom::Current(0))?;
            Err(binread::Error::Custom {
                pos,
                err: Box::new(e),
            })
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlayerInfo {
    pub max: u32,
    pub online: u32,
    #[serde(default)]
    pub sample: Vec<PlayerSample>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerSample {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct DescriptionInfo {
    pub text: String,
}
