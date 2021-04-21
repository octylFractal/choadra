use attohttpc::body::Json;
use serde::Serialize;

use crate::mojang::error::HttpChoadraResult;

#[derive(Serialize)]
pub struct JoinSession {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "selectedProfile")]
    pub selected_profile: String,
    #[serde(rename = "serverId")]
    pub server_id: String,
}

impl JoinSession {
    pub fn exchange(&self) -> HttpChoadraResult<()> {
        attohttpc::post("https://sessionserver.mojang.com/session/minecraft/join")
            .header("Content-type", "application/json")
            .body(Json(self))
            .send()?
            .error_for_status()?;

        Ok(())
    }
}
