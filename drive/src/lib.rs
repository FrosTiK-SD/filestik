extern crate google_drive3 as drive;
use std::sync::{Arc, Mutex};

use anyhow::{Error, Ok, Result};
use drive::{
    api::{File, FileList},
    hyper,
    hyper_rustls::{self, HttpsConnector},
    oauth2::authenticator::Authenticator,
    DriveHub,
};
use fs::{archive::archive_v2, cache::CacheManager, FileManager};
use futures::future::join_all;
use interface::CreateFileStruct;
use tokio::spawn;
use upload::upload_batch;

pub mod create;
pub mod download;
pub mod interface;
pub mod link;
pub mod list;
pub mod upload;

#[derive(Clone)]
pub struct DriveManager {
    pub hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
    pub cache: Arc<Mutex<CacheManager>>,
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
            cache: Arc::new(Mutex::new(CacheManager::new())),
        })
    }

    pub async fn get_file_list(
        &self,
        query: Option<&str>,
        page_token: Option<&str>,
        custom_fields: Option<&str>,
    ) -> Result<FileList, Error> {
        Ok(
            list::get_file_list(Arc::new(self.clone()), query, page_token, custom_fields)
                .await
                .unwrap(),
        )
    }

    pub async fn upload_files(
        &self,
        upload_files_req: Vec<CreateFileStruct>,
    ) -> Result<Vec<Option<String>>> {
        upload_batch(Arc::new(self.clone()), upload_files_req).await
    }

    pub async fn create_shortcut(&self, file_id: String, parent_id: String) -> Result<File, Error> {
        Ok(create::shortcut(self.hub.clone(), file_id, parent_id)
            .await
            .unwrap())
    }

    pub async fn download_file(&self, url: &str) -> Result<Vec<FileManager>> {
        let response = download::universal(Arc::new(self.clone()), url)
            .await
            .unwrap();
        let downloaded_files = response.lock().unwrap().clone();
        archive_v2(downloaded_files.clone()).await;
        spawn(CacheManager::cleanup_and_store_in_cache(
            downloaded_files.clone(),
            self.cache.clone(),
        ));
        Ok(downloaded_files)
    }

    pub fn get_call_hash(
        call_type: &str,
        query: String,
        page_token: String,
        custom_fields: String,
    ) -> String {
        format!(
            "filesSTiK | {} | {} | {} | {}",
            call_type.to_string(),
            query,
            page_token,
            custom_fields
        )
    }
}
