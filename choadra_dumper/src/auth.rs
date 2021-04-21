use serde::{Deserialize, Serialize};

use choadra::client::Credentials;
use choadra::error::{ChoadraError, ChoadraResult};
use choadra::mojang::auth::{Agent, Authenticate};
use choadra::mojang::refresh::Refresh;
use choadra::mojang::validate::Validate;
use choadra::protocol::datatype::uuid::UUID;

use crate::config::APP_CONFIG;

pub fn authenticate_if_needed(
    offline_mode: bool,
    username: String,
) -> ChoadraResult<(String, Option<Credentials>)> {
    if offline_mode {
        return Ok((username, None));
    }

    let client_token = generate_or_load_client_token()?;

    let file = APP_CONFIG.join("saved_credentials.json");
    if file.exists() {
        let mut saved: SavedCredentials = serde_json::from_reader(std::fs::File::open(&file)?)
            .map_err(|e| ChoadraError::InvalidState {
                msg: format!("Failed to read from saved creds: {:?}", e),
            })?;
        let valid = Validate {
            access_token: saved.token.clone(),
            client_token: client_token.clone(),
        }
        .exchange()?;
        if !valid {
            let response = Refresh {
                access_token: saved.token.clone(),
                client_token: client_token.clone(),
            }
            .exchange()?;
            saved = SavedCredentials {
                username: response.selected_profile.name,
                token: response.access_token,
                profile: response.selected_profile.id,
            };
        }
        Ok(saved.into())
    } else {
        let new_creds = ask_for_credentials(username, client_token)?;
        std::fs::create_dir_all(&*APP_CONFIG)?;
        serde_json::to_writer(std::fs::File::create(&file)?, &new_creds).map_err(|e| {
            ChoadraError::InvalidState {
                msg: format!("Failed to write to saved creds: {:?}", e),
            }
        })?;
        Ok(new_creds.into())
    }
}

fn ask_for_credentials(username: String, client_token: String) -> ChoadraResult<SavedCredentials> {
    let password = rpassword::prompt_password_stderr("Password: ").map_err(|_| {
        ChoadraError::InvalidState {
            msg: format!("Needed a password, but none given."),
        }
    })?;

    let response = Authenticate {
        agent: Agent {
            name: "Minecraft".to_string(),
            version: 1,
        },
        username,
        password,
        client_token: Some(client_token),
    }
    .exchange()?;

    Ok(SavedCredentials {
        username: response.selected_profile.name,
        token: response.access_token,
        profile: response.selected_profile.id,
    })
}

fn generate_or_load_client_token() -> ChoadraResult<String> {
    let file = APP_CONFIG.join("client_token.txt");
    if file.exists() {
        std::fs::read_to_string(file).map_err(ChoadraError::from)
    } else {
        let uuid = UUID::new_v4().to_string();
        std::fs::create_dir_all(&*APP_CONFIG)?;
        std::fs::write(file, uuid.as_bytes())?;
        Ok(uuid)
    }
}

#[derive(Deserialize, Serialize)]
pub struct SavedCredentials {
    pub username: String,
    pub token: String,
    pub profile: String,
}

impl From<SavedCredentials> for (String, Option<Credentials>) {
    fn from(creds: SavedCredentials) -> Self {
        (
            creds.username,
            Some(Credentials {
                token: creds.token,
                profile: creds.profile,
            }),
        )
    }
}
