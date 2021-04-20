use aes::cipher::generic_array::GenericArray;
use aes::cipher::NewStreamCipher;
use aes::Aes128;
use cfb8::Cfb8;
use rand::{thread_rng, RngCore};
use rsa::{PaddingScheme, PublicKey, RSAPublicKey};

use crate::client::def::EncryptableStream;
use crate::client::{ChoadraClient, Login, Play};
use crate::error::{ChoadraError, ChoadraResult};
use crate::protocol::aes::AesStream;
use crate::protocol::login::c2s::{EncryptionResponse, LoginStart};
use crate::protocol::login::s2c::S2CLoginPacket;

impl ChoadraClient<Login> {
    pub fn login(mut self, username: String) -> ChoadraResult<ChoadraClient<Play>> {
        // send start packet
        self.write_c2s_packet(LoginStart { username })?;

        // read request
        let request = {
            match self.read_s2c_packet()? {
                S2CLoginPacket::EncryptionRequest(r) => r,
                p => {
                    return Err(ChoadraError::ServerError {
                        msg: format!("Got {:?} instead of an EncryptionRequest", p),
                    });
                }
            }
        };
        let public_key = RSAPublicKey::from_pkcs1(&request.public_key)?;
        // generate secret, prep AES
        let secret = {
            let mut secret = [0u8; 16];
            thread_rng().fill_bytes(&mut secret);
            GenericArray::from(secret)
        };

        // send response
        let encrypted_shared_secret =
            public_key.encrypt(&mut thread_rng(), PaddingScheme::PKCS1v15Encrypt, &secret)?;
        let encrypted_verify_token = public_key.encrypt(
            &mut thread_rng(),
            PaddingScheme::PKCS1v15Encrypt,
            &request.verify_token,
        )?;
        self.write_c2s_packet(EncryptionResponse {
            shared_secret: encrypted_shared_secret,
            verify_token: encrypted_verify_token,
        })?;
        self.writer = EncryptableStream::Encrypted(AesStream::new(
            self.writer.into_inner(),
            Cfb8::<Aes128>::new_var(&secret, &secret).expect("Bug in secret gen"),
        ));
        self.reader = EncryptableStream::Encrypted(AesStream::new(
            self.reader.into_inner(),
            Cfb8::<Aes128>::new_var(&secret, &secret).expect("Bug in secret gen"),
        ));

        Ok(self.into_other_variant(Play))
    }
}
