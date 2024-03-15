extern crate google_drive3 as drive;
use std::sync::Arc;

use anyhow::{Error, Ok, Result};
use drive::{
    api::{File, FileList},
    hyper,
    hyper_rustls::{self, HttpsConnector},
    oauth2::authenticator::Authenticator,
    DriveHub,
};
use fs::{archive::archive_v2, cache::CacheManager, compression::compress, FileManager};
use tokio::spawn;

mod create;
mod download;
pub mod link;
mod list;

#[derive(Clone)]
pub struct DriveManager {
    pub hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    pub cache: CacheManager,
}

impl DriveManager {
    pub fn new(
        connector: Authenticator<HttpsConnector<drive::hyper::client::HttpConnector>>,
    ) -> Result<Self> {
        let hub = Arc::new(DriveHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .build(),
            ),
            connector,
        ));

        Ok(Self {
            hub,
            cache: CacheManager::new(),
        })
    }

    pub async fn get_file_list(
        &self,
        query: Option<&str>,
        page_token: Option<&str>,
        custom_fields: Option<&str>,
    ) -> Result<FileList, Error> {
        Ok(
            list::get_file_list(self.hub.clone(), query, page_token, custom_fields)
                .await
                .unwrap(),
        )
    }

    pub async fn create_shortcut(&self, file_id: String, parent_id: String) -> Result<File, Error> {
        Ok(create::shortcut(self.hub.clone(), file_id, parent_id)
            .await
            .unwrap())
    }

    pub async fn download_file(&self, url: &str) -> Result<Vec<FileManager>> {
        let response = download::universal(self.hub.clone(), url).await.unwrap();
        let compressed_response = compress(response).await;
        let downloaded_files = compressed_response.lock().unwrap().clone();
        archive_v2(downloaded_files.clone()).await;
        spawn(CacheManager::store_in_cache(downloaded_files.clone()));
        Ok(downloaded_files)
    }
}
