use std::sync::Arc;

use anyhow::{Error, Result};
use drive::{api::FileList, hyper_rustls::HttpsConnector, DriveHub};

pub async fn get_file_list(
    hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    query: Option<&str>,
    page_token: Option<&str>,
    custom_fields: Option<&str>,
) -> Result<FileList, Error> {
    let (q, pt, f): (&str, &str, &str) = (
        query.unwrap_or_default(),
        page_token.unwrap_or_default(),
        custom_fields.unwrap_or(
            "files/shortcutDetails, files/mimeType, files/name, files/id, files/fileExtension, files/headRevisionId",
        ),
    );

    let (_, file_list) = hub
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

    Ok(file_list)
}
