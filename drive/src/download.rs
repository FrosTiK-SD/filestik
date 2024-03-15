use std::sync::{Arc, Mutex};

use ::fs::FileManager;
use anyhow::{Ok, Result};
use async_recursion::async_recursion;
use drive::{hyper_rustls::HttpsConnector, DriveHub};
use futures::future::join_all;
use google_drive3::{api::File, hyper::body::HttpBody};

use tokio::spawn;

use crate::{link::Link, list::get_file_list, DriveManager};

async fn download_mormal_file(
    drive: Arc<DriveManager>,
    file_metadata: File,
    downloaded_files: Arc<Mutex<Vec<FileManager>>>,
) {
    let file_manager = FileManager::new(
        file_metadata.clone(),
        drive.cache.clone(),
        "tmp/files".to_string(),
    );

    // Download if not already cached
    if !file_manager.is_cached.clone() {
        // Get the file contents
        let (response, _) = drive
            .hub
            .clone()
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
        file_manager.write_file(file_bytes).await.unwrap();
        println!("DOWNLOADED FILE - {:#?}", file_metadata.id.unwrap());
    }

    // Append to list of downloaded files
    downloaded_files.lock().unwrap().push(file_manager);
}

async fn download_workspace_file(
    drive: Arc<DriveManager>,
    file_metadata: File,
    downloaded_files: Arc<Mutex<Vec<FileManager>>>,
) {
    let file_manager = FileManager::new(
        file_metadata.clone(),
        drive.cache.clone(),
        "tmp/files".to_string(),
    );

    // Only download if not already cached
    if !file_manager.is_cached.clone() {
        let new_mime_type = file_manager.mime_type.clone();

        if new_mime_type.is_empty() {
            println!("File format not currently supported by FilesTiK")
        }

        // Get the file contents
        let response = drive
            .hub
            .clone()
            .files()
            .export(
                file_metadata.id.clone().unwrap().as_str(),
                new_mime_type.as_str(),
            )
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
        file_manager.write_file(file_bytes).await.unwrap();
        println!("DOWNLOADED FILE - {:#?}", file_metadata.id.unwrap());
    }

    // Append to list of downloaded files
    downloaded_files.lock().unwrap().push(file_manager);
}

#[async_recursion]
async fn download_folder(
    drive: Arc<DriveManager>,
    folder_id: String,
    page_token: Option<String>,
    downloaded_files: Arc<Mutex<Vec<FileManager>>>,
) {
    let filter = format!("'{}' in parents and trashed=false", folder_id);
    let file_list = get_file_list(
        drive.hub.clone(),
        Some(filter.as_str()),
        Some(page_token.unwrap_or_default().as_str()),
        None,
    )
    .await
    .unwrap();

    let mut thread_handlers = vec![];

    for f in file_list.files.unwrap() {
        thread_handlers.push(spawn(segregate_downloads(
            drive.clone(),
            f,
            downloaded_files.clone(),
        )));
    }

    if !file_list.next_page_token.is_none() {
        download_folder(
            drive,
            folder_id,
            file_list.next_page_token,
            downloaded_files,
        )
        .await;
    }

    join_all(thread_handlers).await;
}

#[async_recursion]
async fn segregate_downloads(
    drive: Arc<DriveManager>,
    file_metadata: File,
    downloaded_files: Arc<Mutex<Vec<FileManager>>>,
) {
    match file_metadata.mime_type.clone().unwrap() {
        // Handle folders
        mime_type if mime_type == String::from("application/vnd.google-apps.folder") => {
            spawn(download_folder(
                drive.clone(),
                file_metadata.id.clone().unwrap(),
                None,
                downloaded_files.clone(),
            ))
            .await
            .unwrap();
            return;
        }

        // Handle shortcuts
        mime_type if mime_type == "application/vnd.google-apps.shortcut" => {
            let original_file = metadata(
                drive.hub.clone(),
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

            segregate_downloads(drive.clone(), original_file, downloaded_files).await;
        }

        // Handle workspace files
        mime_type if mime_type.starts_with("application/vnd.google-apps") => {
            spawn(download_workspace_file(
                drive.clone(),
                file_metadata.clone(),
                downloaded_files.clone(),
            ))
            .await
            .unwrap();
            return;
        }

        // Handle non-Workspace files
        _ => {
            spawn(download_mormal_file(
                drive.clone(),
                file_metadata,
                downloaded_files.clone(),
            ))
            .await
            .unwrap();
            return;
        }
    }
}

pub async fn metadata(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    file_id: &str,
    custom_fields: Option<&str>,
) -> Result<File> {
    let fields = custom_fields
        .unwrap_or("shortcutDetails, mimeType, name, id, fileExtension, headRevisionId");
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
    drive: Arc<DriveManager>,
    url: &str,
) -> Result<Arc<Mutex<Vec<FileManager>>>> {
    let link = Link::new(url.to_string());
    let downloaded_files = Arc::new(Mutex::new(Vec::new()));

    // Get the metadata
    let file_metadata = metadata(drive.hub.clone(), &link.id, None).await.unwrap();

    segregate_downloads(drive.clone(), file_metadata, downloaded_files.clone()).await;

    Ok(downloaded_files)
}
