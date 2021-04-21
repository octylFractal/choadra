use attohttpc::body::Json;
use serde::{Deserialize, Serialize};

use crate::mojang::error::{HttpChoadraError, HttpChoadraResult};

#[derive(Serialize)]
pub struct Authenticate {
    pub agent: Agent,
    pub username: String,
    pub password: String,
    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,
}

#[derive(Serialize)]
pub struct Agent {
    pub name: String,
    pub version: u32,
}

impl Authenticate {
    pub fn exchange(&self) -> HttpChoadraResult<AuthenticateResponse> {
        let resp = attohttpc::post("https://authserver.mojang.com/authenticate")
            .header("User-agent", "choadra")
            .header("Content-type", "application/json")
            .body(Json(self))
            .send()?;
        if resp.is_success() {
            return Ok(resp.json()?);
        }
        return Err(HttpChoadraError::ExtractedHttpError(format!(
            "Code: {}, Body: {}",
            resp.status(),
            resp.text()?
        )));
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthenticateResponse {
    #[serde(rename = "clientToken")]
    pub client_token: String,
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "selectedProfile")]
    pub selected_profile: Profile,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub name: String,
    pub id: String,
}
