use std::collections::VecDeque;

use aes::cipher::generic_array::GenericArray;
use aes::cipher::NewStreamCipher;
use aes::Aes128;
use cfb8::Cfb8;
use rand::{thread_rng, RngCore};
use rsa::{PaddingScheme, PublicKey, RSAPublicKey};
use sha1::{Digest, Sha1};

use crate::client::def::{Credentials, EncryptableStream};
use crate::client::{ChoadraClient, Login, Play};
use crate::error::{ChoadraError, ChoadraResult};
use crate::mojang::sessionserver::JoinSession;
use crate::protocol::aes::AesStream;
use crate::protocol::login::c2s::{EncryptionResponse, LoginStart};
use crate::protocol::login::s2c::{EncryptionRequest, S2CLoginPacket};
use crate::protocol::util::mojang_hex_encode;

impl ChoadraClient<Login> {
    /// Perform login.
    /// `credentials` can be None to fail if the server is in online-mode.
    pub fn login(
        mut self,
        username: String,
        mut credentials: Option<Credentials>,
    ) -> ChoadraResult<ChoadraClient<Play>> {
        // send start packet
        self.write_c2s_packet(LoginStart {
            username: username.clone(),
        })?;

        let mut encrypted = false;
        // read reply
        loop {
            match self.read_s2c_packet()? {
                S2CLoginPacket::EncryptionRequest(r) => {
                    if encrypted {
                        return Err(ChoadraError::ServerError {
                            msg: format!("Server tried to encrypt when already encrypted"),
                        });
                    }
                    let credentials =
                        credentials
                            .take()
                            .ok_or_else(|| ChoadraError::InvalidState {
                                msg: format!("A token is needed, but None was given"),
                            })?;
                    self = self.handle_encryption_flow(r, credentials)?;
                    encrypted = true;
                    // Fallthrough to handle set compression / login success
                }
                S2CLoginPacket::SetCompression(sc) => {
                    self.packet_read_state
                        .set_compression_threshold(sc.threshold);
                    self.packet_write_state
                        .set_compression_threshold(sc.threshold);
                    // Fallthrough to handle login success
                }
                S2CLoginPacket::LoginSuccess(ls) => {
                    if ls.username != username {
                        return Err(ChoadraError::ServerError {
                            msg: format!(
                                "Server replied with {} as the username, not {}",
                                ls.username, username
                            ),
                        });
                    }
                    return Ok(self.into_other_variant(Play {
                        username: ls.username,
                        uuid: ls.uuid,
                        really_playing: false,
                        packet_queue: VecDeque::new(),
                    }));
                }
                p => {
                    return Err(ChoadraError::ServerError {
                        msg: format!(
                            "Got {:?} instead of an EncryptionRequest/SetCompression/LoginSuccess",
                            p
                        ),
                    });
                }
            }
        }
    }

    fn handle_encryption_flow(
        mut self,
        request: EncryptionRequest,
        credentials: Credentials,
    ) -> ChoadraResult<Self> {
        let public_key = RSAPublicKey::from_pkcs8(&request.public_key)?;
        // generate secret
        let secret = {
            let mut secret = [0u8; 16];
            thread_rng().fill_bytes(&mut secret);
            GenericArray::from(secret)
        };

        // initiate session
        let server_id = {
            let mut hasher = Sha1::new();
            hasher.update(request.server_id.as_bytes());
            hasher.update(&secret);
            hasher.update(&request.public_key);
            mojang_hex_encode(&hasher.finalize())
        };

        JoinSession {
            access_token: credentials.token,
            selected_profile: credentials.profile,
            server_id,
        }
        .exchange()?;

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

        Ok(self)
    }
}
