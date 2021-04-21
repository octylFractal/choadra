use attohttpc::body::Json;
use attohttpc::StatusCode;
use serde::Serialize;

use crate::mojang::error::{HttpChoadraError, HttpChoadraResult};

#[derive(Serialize)]
pub struct Validate {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "clientToken")]
    pub client_token: String,
}

impl Validate {
    pub fn exchange(&self) -> HttpChoadraResult<bool> {
        let res = attohttpc::post("https://authserver.mojang.com/validate")
            .header("Content-type", "application/json")
            .body(Json(self))
            .send()?;

        match res.status() {
            StatusCode::NO_CONTENT => Ok(true),
            StatusCode::FORBIDDEN => Ok(false),
            s => Err(HttpChoadraError::AttoHttpClientError(
                attohttpc::ErrorKind::StatusCode(s).into(),
            )),
        }
    }
}
