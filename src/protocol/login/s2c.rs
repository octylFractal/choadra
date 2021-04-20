use binread::derive_binread;

use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::chat::parse_chat;
use crate::protocol::datatype::identifier::Identifier;
use crate::protocol::datatype::string::parse_string;
use crate::protocol::datatype::uuid::{parse_uuid, UUID};
use crate::protocol::datatype::varint::parse_varint;
use crate::protocol::util::parse_until_eof;

#[derive_binread]
#[br(import(id: Int))]
#[derive(Debug)]
pub enum S2CLoginPacket {
    #[br(pre_assert(id == 0x00))]
    Disconnect(Disconnect),
    #[br(pre_assert(id == 0x01))]
    EncryptionRequest(EncryptionRequest),
    #[br(pre_assert(id == 0x02))]
    LoginSuccess(LoginSuccess),
    #[br(pre_assert(id == 0x03))]
    SetCompression(SetCompression),
    #[br(pre_assert(id == 0x04))]
    LoginPluginRequest(LoginPluginRequest),
}

#[derive_binread]
#[derive(Debug)]
pub struct Disconnect {
    #[br(parse_with = parse_chat)]
    pub reason: String,
}

#[derive_binread]
#[derive(Debug)]
pub struct EncryptionRequest {
    #[br(parse_with = parse_string, args(Some(20)))]
    pub server_id: String,
    #[br(temp, parse_with = parse_varint)]
    pub public_key_length: Int,
    #[br(count = public_key_length)]
    pub public_key: Vec<u8>,
    #[br(temp, parse_with = parse_varint)]
    pub verify_token_length: Int,
    #[br(count = verify_token_length)]
    pub verify_token: Vec<u8>,
}

#[derive_binread]
#[derive(Debug)]
pub struct LoginSuccess {
    #[br(parse_with = parse_uuid)]
    pub uuid: UUID,
    #[br(parse_with = parse_string, args(Some(16)))]
    pub username: String,
}

#[derive_binread]
#[derive(Debug)]
pub struct SetCompression {
    #[br(parse_with = parse_varint, map = |x: Int| if x <= 0 { None } else { Some(x) })]
    pub threshold: Option<Int>,
}

#[derive_binread]
#[derive(Debug)]
pub struct LoginPluginRequest {
    #[br(parse_with = parse_varint)]
    pub message_id: Int,
    pub channel: Identifier,
    #[br(parse_with = parse_until_eof)]
    pub data: Vec<u8>,
}
