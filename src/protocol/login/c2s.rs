use std::io::Write;

use crate::protocol::c2s::IdentityPacket;
use crate::protocol::datatype::aliases::Int;
use crate::protocol::datatype::varint::VarInt;
use crate::protocol::datatype::writeable::Writeable;

#[derive(Debug)]
pub struct LoginStart {
    pub username: String,
}

impl IdentityPacket for LoginStart {
    const ID: Int = 0x00;
}

impl Writeable for LoginStart {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        self.username.write_to(write, (Some(16),))?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl IdentityPacket for EncryptionResponse {
    const ID: Int = 0x01;
}

impl Writeable for EncryptionResponse {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        VarInt(self.shared_secret.len() as Int).write_to(write, ())?;
        write.write_all(&self.shared_secret)?;
        VarInt(self.verify_token.len() as Int).write_to(write, ())?;
        write.write_all(&self.verify_token)?;

        Ok(())
    }
}
#[derive(Debug)]
pub struct LoginPluginResponse {
    pub message_id: Int,
    /// If data is None, will set successful to false
    pub data: Option<Vec<u8>>,
}

impl IdentityPacket for LoginPluginResponse {
    const ID: Int = 0x02;
}

impl Writeable for LoginPluginResponse {
    type Args = ();

    fn write_to<W: Write>(&self, write: &mut W, _: Self::Args) -> Result<(), std::io::Error> {
        VarInt(self.message_id).write_to(write, ())?;
        // Write successful
        self.data.is_some().write_to(write, ())?;
        if let Some(data) = &self.data {
            write.write_all(&data)?;
        }

        Ok(())
    }
}
