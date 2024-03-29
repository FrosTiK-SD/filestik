use std::{
    fs,
    sync::{Arc, Mutex},
};

use anyhow::{Ok, Result};
use cache::CacheManager;
use compression::compress;
use google_drive3::{api::File, hyper::body::Bytes};

pub mod archive;
pub mod cache;
pub mod compression;

pub static CACHE_KEY_STORE_PATH: &str = "tmp/.cache/keyStore.csv";
pub static CACHE_FILES_PATH: &str = "tmp/.cache/files";
pub static TMP_BASE_PATH: &str = "tmp";
pub static TMP_FILES_UNCOMPRESSED_BASE_PATH: &str = "tmp/files";
pub static TMP_FILES_COMPRESSED_BASE_PATH: &str = "tmp/compressed";
pub static TMP_FILES_OUTPUT_BASE_PATH: &str = "tmp/output";
pub static TMP_CACHE_PATH: &str = "tmp/.cache";

#[derive(Clone)]
pub struct FileManager {
    pub file: File,
    pub base_path: String,
    pub file_name: String,
    pub mime_type: String,
    pub ext: String,
    pub compressed_file_path: String,
    pub cache_manager: Arc<Mutex<CacheManager>>,
    pub cached_path: String,
    pub is_cached: bool,
}

impl FileManager {
    pub fn new(file: File, cache_manager: Arc<Mutex<CacheManager>>, base_path: String) -> Self {
        let (mime_type, ext) = Self::get_mime_type_and_ext(file.clone());

        let mut file_manager = Self {
            file: file.clone(),
            base_path,
            file_name: Self::get_file_name(file.clone(), ext.clone()),
            mime_type,
            ext,
            compressed_file_path: String::new(),
            cache_manager,
            is_cached: false,
            cached_path: String::new(),
        };

        file_manager.sync_cache(file);

        file_manager
    }

    fn sync_cache(&mut self, file: File) {
        let cache_manager = self.cache_manager.lock().unwrap();
        let revision_map = cache_manager.store.get(file.id.unwrap().as_str());
        if revision_map.is_none() {
            return;
        }

        let rm_unwrap = revision_map.unwrap();
        let default_val = String::from("");
        let cached_file_path = rm_unwrap
            .get(self.get_file_revision_id().as_str())
            .unwrap_or(&default_val);

        if !cached_file_path.is_empty() {
            self.cached_path = cached_file_path.to_string();
            self.is_cached = true
        }

        return;
    }

    // Creates the file name with accurate extension
    fn get_file_name(file: File, ext: String) -> String {
        let name = file.name.clone().unwrap();

        let mut file_name_parts = name.split(".").collect::<Vec<&str>>();
        // Remove the existing extension (if any)
        if file_name_parts.len() > 1 {
            file_name_parts.pop();
        }

        // Add the appropriate extension
        file_name_parts.push(ext.as_str());

        return file_name_parts.join(".").to_string();
    }

    // Calculates what should be the mime_type based on the documentation
    fn get_mime_type_and_ext(file: File) -> (String, String) {
        match file.mime_type.clone().unwrap().as_str() {
            "application/vnd.google-apps.spreadsheet" => (
                String::from("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                String::from("xlsx"),
            ),
            "application/vnd.google-apps.document" => {
                (String::from("application/pdf"), String::from("pdf"))
            }
            "application/vnd.google-apps.presentation" => (
                String::from(
                    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                ),
                String::from("pptx"),
            ),
            "application/vnd.google-apps.drawing" => {
                (String::from("application/pdf"), String::from("pdf"))
            }
            "application/vnd.google-apps.script" => (
                String::from("application/vnd.google-apps.script+json"),
                String::from("json"),
            ),
            mime_type if !mime_type.starts_with("application/vnd.google-apps") => {
                (mime_type.to_string(), file.file_extension.unwrap())
            }
            _ => (String::new(), String::new()),
        }
    }

    // Returns the original target path of the file
    pub fn get_target_path(&self) -> String {
        format!("{}/{}", self.base_path, self.file_name)
    }

    // Calculates what should be the location of the compressed files
    pub fn get_compressed_target_path(&self) -> String {
        let target_path = self.get_target_path();
        let mut target_path_parts = target_path.split("/").collect::<Vec<_>>();
        target_path_parts[1] = "compressed";

        target_path_parts.join("/")
    }

    // Returns the compressed target path if exists or returns the actual target path
    pub fn get_optimal_target_path(&self) -> String {
        if self.compressed_file_path.is_empty() {
            if self.is_cached {
                self.cached_path.clone()
            } else {
                self.get_target_path()
            }
        } else {
            self.compressed_file_path.clone()
        }
    }

    // Checks the target path and the base path and returns the relative path
    pub fn get_relative_path(&self) -> String {
        let target_path = self.get_target_path();
        let base_path_parts = self.base_path.split("/").collect::<Vec<&str>>();
        let target_path_parts = target_path.split("/").collect::<Vec<&str>>();

        let mut relative_path_parts = Vec::new();
        for path_idx in base_path_parts.len()..target_path_parts.len() {
            relative_path_parts.push(target_path_parts[path_idx]);
        }

        return relative_path_parts.join("/");
    }

    // Write file to fs
    pub async fn write_file(&self, content: Bytes) -> Result<()> {
        fs::create_dir_all(self.base_path.as_str()).unwrap();
        fs::write(self.get_target_path(), &content).unwrap();
        compress(Arc::new(Mutex::new(vec![self.clone()]))).await;
        Ok(())
    }

    pub fn get_file_revision_id(&self) -> String {
        self.file
            .head_revision_id
            .clone()
            .unwrap_or("_".to_string())
    }
}
