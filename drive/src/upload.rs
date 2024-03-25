use std::{
    fs,
    sync::{Arc, Mutex},
};

use anyhow::{Ok, Result};
use drive::api::{File, Permission};
use futures::future::join_all;
use tokio::spawn;

use crate::{interface::CreateFileStruct, DriveManager};

pub async fn upload_file(
    drive: Arc<DriveManager>,
    upload_file: CreateFileStruct,
    link_store: Arc<Mutex<Vec<Option<String>>>>,
) -> Result<()> {
    let mime_type_string = format!("{}", upload_file.mime_type.clone().unwrap());
    let file = File {
        name: Some(upload_file.name.clone()),
        mime_type: Some(mime_type_string),
        parents: Some(upload_file.parents.clone()),
        ..File::default()
    };

    let (_, file) = if upload_file.file_id.clone().is_none() {
        println!("UPLOADING");
        drive
            .hub
            .files()
            .create(file)
            .supports_all_drives(true)
            .param("fields", "webViewLink, id")
            .ocr_language("en")
            .upload(upload_file.content, upload_file.mime_type.unwrap())
            .await
            .unwrap()
    } else {
        println!("UPDATING");
        drive
            .hub
            .files()
            .update(file, upload_file.file_id.clone().unwrap().as_str())
            .supports_all_drives(true)
            .param("fields", "webViewLink, id")
            .param("newRevision", "true")
            .ocr_language("en")
            .upload(upload_file.content, upload_file.mime_type.unwrap())
            .await
            .unwrap()
    };

    // Create permissions for view access
    drive
        .hub
        .permissions()
        .create(
            Permission {
                type_: Some(String::from("anyone")),
                role: Some(String::from("reader")),
                ..Permission::default()
            },
            file.id.clone().unwrap().as_str(),
        )
        .doit()
        .await
        .unwrap();

    link_store.lock().unwrap().push(file.web_view_link);

    Ok(())
}

pub async fn upload_batch(
    drive: Arc<DriveManager>,
    upload_files: Vec<CreateFileStruct>,
) -> Result<(Vec<Option<String>>)> {
    let mut thread_handlers = vec![];
    let link_store = Arc::new(Mutex::new(vec![]));

    for file_metadata in upload_files {
        thread_handlers.push(spawn(upload_file(
            drive.clone(),
            file_metadata,
            link_store.clone(),
        )))
    }

    join_all(thread_handlers).await;

    let link_store_populated = link_store.lock().unwrap().clone();

    Ok(link_store_populated)
}
