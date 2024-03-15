use csv::{ReaderBuilder, StringRecord, Writer};
use dashmap::DashMap;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

use crate::{
    FileManager, CACHE_FILES_PATH, CACHE_KEY_STORE_PATH, TMP_BASE_PATH, TMP_CACHE_PATH,
    TMP_FILES_COMPRESSED_BASE_PATH, TMP_FILES_OUTPUT_BASE_PATH, TMP_FILES_UNCOMPRESSED_BASE_PATH,
};

#[derive(Debug, Clone)]
pub struct CacheManager {
    // HashMap<FileID -> RevisionID -> Path>
    pub store: DashMap<String, HashMap<String, String>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self::run_fs_checks();
        let mut cache_manager = Self {
            store: DashMap::new(),
        };

        cache_manager.initialize();
        cache_manager
    }

    pub fn run_fs_checks() {
        fs::create_dir_all(TMP_BASE_PATH).unwrap();
        fs::create_dir_all(TMP_CACHE_PATH).unwrap();
        fs::create_dir_all(CACHE_FILES_PATH).unwrap();
        fs::create_dir_all(TMP_FILES_COMPRESSED_BASE_PATH).unwrap();
        fs::create_dir_all(TMP_FILES_UNCOMPRESSED_BASE_PATH).unwrap();
        fs::create_dir_all(TMP_FILES_OUTPUT_BASE_PATH).unwrap();

        // Check if keyStore exists
        if !Path::new(CACHE_KEY_STORE_PATH).exists() {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(CACHE_KEY_STORE_PATH)
                .unwrap();

            file.write("file_id,revision_id,path,file_name,timestamp\n".as_bytes())
                .unwrap();
        }
    }

    fn parse_row_in_memory(&mut self, record: StringRecord) {
        let file_id = record.get(0).clone().unwrap().to_string();
        let revision_id = record.get(1).clone().unwrap().to_string();
        let target_path = record.get(2).clone().unwrap().to_string();

        self.store
            .entry(file_id)
            .and_modify(|revision_map| {
                revision_map.insert(revision_id.clone(), target_path.clone());
            })
            .or_insert(HashMap::from([(revision_id, target_path)]));
    }

    pub fn initialize(&mut self) {
        let file = File::open(CACHE_KEY_STORE_PATH).expect("Cant open file");

        let mut rdr = ReaderBuilder::new().delimiter(b',').from_reader(file);

        for result in rdr.records() {
            let record = result.unwrap();
            self.parse_row_in_memory(record);
        }
    }

    pub fn get_cache_file_path(fm: FileManager) -> String {
        format!(
            "{}/{}_{}.{}",
            CACHE_FILES_PATH,
            fm.file.id.clone().unwrap(),
            fm.file.head_revision_id.clone().unwrap_or("_".to_string()),
            fm.ext.clone()
        )
    }

    pub async fn store_in_cache(fm_list: Vec<FileManager>) {
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(CACHE_KEY_STORE_PATH)
            .unwrap();
        let mut wtr = Writer::from_writer(file);

        for fm in fm_list {
            let cache_file_path = Self::get_cache_file_path(fm.clone());

            fs::copy(fm.get_optimal_target_path(), cache_file_path.clone()).unwrap();

            // Only replace if the cache file doesnot exist -> Otherwise dont do anything

            wtr.write_record(&[
                fm.file.id.clone().unwrap(),
                fm.file.head_revision_id.clone().unwrap_or("_".to_string()),
                cache_file_path,
                fm.file_name.replace(",", ""),
                chrono::offset::Local::now().to_string(),
            ])
            .unwrap();
        }
    }
}
