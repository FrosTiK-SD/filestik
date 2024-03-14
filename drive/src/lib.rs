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
use url::Url;

mod create;
mod download;
mod list;

#[derive(Debug)]
pub struct Link {
    pub url: String,
    pub id: String,
}

impl Link {
    pub fn new(url: String) -> Self {
        let parsed_url = Url::parse(url.as_str());
        let mut id = url.clone();

        if parsed_url.is_ok() {
            let unwrapped_parsed_url = parsed_url.unwrap();
            let path_split = unwrapped_parsed_url
                .path_segments()
                .map(|c| c.collect::<Vec<_>>())
                .unwrap();

            if path_split.len() >= 3 {
                id = path_split.get(2).unwrap().to_string();
            }
        }

        Self {
            url: url.to_string(),
            id,
        }
    }
}

#[derive(Clone)]
pub struct DriveManager {
    pub hub: Arc<DriveHub<HttpsConnector<drive::hyper::client::HttpConnector>>>,
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

        Ok(Self { hub })
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

    pub async fn download_file(&self, url: &str) {
        download::universal(self.hub.clone(), url).await;
    }
}