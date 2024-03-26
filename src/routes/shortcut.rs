use actix_web::{
    post,
    web::{Data, Json},
    Responder, Result,
};
use drive::api::File;
use drive_manager::DriveManager;
use serde::Deserialize;

use super::interface::GenericResponse;

#[derive(Deserialize)]
pub struct CreateShortcutRequestBody {
    pub target: String,
    pub parents: Vec<String>,
}

#[post("/shortcut")]
pub async fn create_shortcut(
    drive_manager: Data<DriveManager>,
    body: Json<CreateShortcutRequestBody>,
) -> Result<impl Responder> {
    let file = drive_manager
        .create_shortcut(body.target.clone(), body.parents.clone(), Some("*"))
        .await
        .unwrap();
    Ok(GenericResponse::<File>::ok(
        "Successfully created shortcut",
        Some(file),
    ))
}
