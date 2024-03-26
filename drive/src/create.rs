use std::{fs, sync::Arc};

use anyhow::{Ok, Result};
use drive::{
    api::{File, FileShortcutDetails},
    hyper_rustls::HttpsConnector,
    DriveHub,
};

pub async fn shortcut(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    file_id: String,
    parent_ids: Vec<String>,
    custom_fields: Option<&str>,
) -> Result<File> {
    let mime_type = "application/vnd.google-apps.shortcut".to_string();
    let file = File {
        mime_type: Some(mime_type.clone()),
        parents: Some(parent_ids),
        shortcut_details: Some(FileShortcutDetails {
            target_id: Some(file_id),
            ..Default::default()
        }),
        ..Default::default()
    };
    let fields = custom_fields.unwrap_or(
        "shortcutDetails, mimeType, name, id, fileExtension, headRevisionId, webViewLink",
    );

    let (_, shortcut) = hub
        .files()
        .create(file)
        .supports_all_drives(true)
        .param("fields", fields)
        .ocr_language("en")
        .upload(
            fs::File::open("shortcut.txt").unwrap(),
            mime_type.parse().unwrap(),
        )
        .await
        .unwrap();

    Ok(shortcut)
}
