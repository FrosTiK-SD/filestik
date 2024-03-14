use std::{fs, sync::Arc};

use anyhow::{Ok, Result};
use async_recursion::async_recursion;
use drive::{hyper_rustls::HttpsConnector, DriveHub};
use google_drive3::{api::File, hyper::body::HttpBody};

use tokio::spawn;

use crate::{list::get_file_list, Link};

async fn download_mormal_file(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    file_metadata: File,
) {
    // Get the file contents
    let (response, _) = hub
        .files()
        .get(file_metadata.id.clone().unwrap().as_str())
        .add_scope("https://www.googleapis.com/auth/drive.readonly")
        .param("alt", "media")
        .supports_all_drives(true)
        .acknowledge_abuse(true)
        .doit()
        .await
        .expect(
            format!(
                "{} | Unable to download",
                file_metadata.name.clone().unwrap()
            )
            .as_str(),
        );

    // Write to disk
    let file_bytes = response.collect().await.unwrap().to_bytes();
    fs::write(file_metadata.name.clone().unwrap(), &file_bytes).expect("Cant write");
}

async fn download_workspace_file(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    file_metadata: File,
) {
    let new_mime_type = match file_metadata.mime_type.clone().unwrap().as_str() {
        "application/vnd.google-apps.spreadsheet" => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        "application/vnd.google-apps.document" => "application/pdf",
        "application/vnd.google-apps.presentation" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        "application/vnd.google-apps.drawing" => "application/pdf",
        "application/vnd.google-apps.script" => "application/vnd.google-apps.script+json",
        _ => "",
    };

    if new_mime_type.is_empty() {
        println!("File format not currently supported by FilesTiK")
    }

    // Get the file contents
    let response = hub
        .files()
        .export(file_metadata.id.clone().unwrap().as_str(), new_mime_type)
        .add_scope("https://www.googleapis.com/auth/drive.readonly")
        .param("alt", "media")
        .doit()
        .await
        .expect(
            format!(
                "{} | Unable to download",
                file_metadata.name.clone().unwrap()
            )
            .as_str(),
        );

    // Write to disk
    let file_bytes = response.collect().await.unwrap().to_bytes();
    fs::write(file_metadata.name.clone().unwrap(), &file_bytes).expect("Cant write");
}

#[async_recursion]
async fn download_folder(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    folder_id: String,
    page_token: Option<String>,
) {
    let filter = format!("'{}' in parents and trashed=false", folder_id);
    let file_list = get_file_list(
        hub.clone(),
        Some(filter.as_str()),
        Some(page_token.unwrap_or_default().as_str()),
        None,
    )
    .await
    .unwrap();

    println!("{:#?}", file_list.files.clone().unwrap().len());

    for f in file_list.files.unwrap() {
        spawn(segregate_downloads(hub.clone(), f));
    }

    if file_list.next_page_token.is_none() {
        return;
    }

    download_folder(hub, folder_id, file_list.next_page_token).await;
}

#[async_recursion]
async fn segregate_downloads(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    file_metadata: File,
) {
    match file_metadata.mime_type.clone().unwrap() {
        // Handle folders
        mime_type if mime_type == String::from("application/vnd.google-apps.folder") => {
            spawn(download_folder(
                hub,
                file_metadata.id.clone().unwrap(),
                None,
            ));
            return;
        }

        // Handle shortcuts
        mime_type if mime_type == "application/vnd.google-apps.shortcut" => {
            let original_file = metadata(
                hub.clone(),
                file_metadata
                    .shortcut_details
                    .unwrap()
                    .target_id
                    .unwrap()
                    .as_str(),
                None,
            )
            .await
            .unwrap();

            segregate_downloads(hub.clone(), original_file).await;
            return;
        }

        // Handle workspace files
        mime_type if mime_type.starts_with("application/vnd.google-apps") => {
            spawn(download_workspace_file(hub.clone(), file_metadata.clone()));
            return;
        }

        // Handle non-Workspace files
        _ => {
            spawn(download_mormal_file(hub.clone(), file_metadata));
            return;
        }
    }
}

pub async fn metadata(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    file_id: &str,
    custom_fields: Option<&str>,
) -> Result<File> {
    let fields = custom_fields.unwrap_or("shortcutDetails, mimeType, name, id, fileExtension");
    let (_, file_metadata) = hub
        .files()
        .get(file_id)
        .param("fields", fields)
        .doit()
        .await
        .unwrap();

    Ok(file_metadata)
}

pub async fn universal(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    url: &str,
) {
    let link = Link::new(url.to_string());

    // Get the metadata
    let file_metadata = metadata(hub.clone(), &link.id, None).await.unwrap();

    segregate_downloads(hub, file_metadata).await;
}
