use std::collections::HashMap;

pub struct CacheManager {
    // HashMap<FileID -> RevisionID -> Path>
    store: HashMap<String, HashMap<String, String>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn initialize(&self) {}
}
