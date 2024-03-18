use std::sync::Arc;

use anyhow::{Error, Ok, Result};
use drive::api::FileList;
use fs::cache::RedisRequest;

use crate::DriveManager;

pub async fn get_file_list(
    drive: Arc<DriveManager>,
    query: Option<&str>,
    page_token: Option<&str>,
    custom_fields: Option<&str>,
) -> Result<FileList, Error> {
    let (q, pt, f): (&str, &str, &str) = (
        query.unwrap_or_default(),
        page_token.unwrap_or_default(),
        custom_fields.unwrap_or(
            "files/shortcutDetails, files/mimeType, files/name, files/id, files/fileExtension, files/headRevisionId, files/webViewLink, nextPageToken",
        ),
    );

    let cache_key =
        DriveManager::get_call_hash("files.list", q.to_string(), pt.to_string(), f.to_string());

    let redis_response = drive
        .cache
        .lock()
        .unwrap()
        .get_from_redis::<RedisRequest<FileList>>(cache_key.clone());

    if redis_response.is_ok() {
        let file_list = redis_response.unwrap().data;
        return Ok(file_list);
    }

    let (_, file_list) = drive
        .hub
        .files()
        .list()
        .q(q)
        .param("fields", f)
        .page_token(pt)
        .include_items_from_all_drives(true)
        .supports_all_drives(true)
        .doit()
        .await
        .expect("Error in fetching files");

    drive.cache.lock().unwrap().set_to_redis(
        cache_key.clone(),
        RedisRequest {
            data: file_list.clone(),
        },
    );

    Ok(file_list)
}
