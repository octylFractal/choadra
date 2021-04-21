use attohttpc::body::Json;
use serde::{Deserialize, Serialize};

use crate::mojang::error::HttpChoadraResult;
use crate::mojang::auth::Profile;

#[derive(Serialize)]
pub struct Refresh {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "clientToken")]
    pub client_token: String,
}

impl Refresh {
    /// Exchange the access token for a new one.
    pub fn exchange(&self) -> HttpChoadraResult<RefreshResponse> {
        Ok(attohttpc::post("https://authserver.mojang.com/refresh")
            .header("Content-type", "application/json")
            .body(Json(self))
            .send()?
            .error_for_status()?
            .json()?)
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "clientToken")]
    pub client_token: String,
    #[serde(rename = "selectedProfile")]
    pub selected_profile: Profile,
}
