use std::sync::{Arc, Mutex};

use futures::future::join_all;
use pdfshrink::gs_command;
use tokio::spawn;

use crate::FileManager;

async fn compress_pdf(
    file: Arc<FileManager>,
    file_idx: usize,
    fm_list: Arc<Mutex<Vec<FileManager>>>,
) {
    let output_path = file.get_compressed_target_path();

    // Dont compress if already cached -> compressed
    if file.is_cached {
        return;
    }

    // We call the get_optimal_target_path() which should return the cached path if present
    let error = gs_command(file.get_optimal_target_path(), output_path.clone()).spawn();
    if error.is_ok() {
        error.unwrap().wait().unwrap();
        fm_list
            .lock()
            .unwrap()
            .get_mut(file_idx)
            .unwrap()
            .compressed_file_path = output_path;
    }
}

pub async fn compress(fm_list: Arc<Mutex<Vec<FileManager>>>) -> Arc<Mutex<Vec<FileManager>>> {
    let mut files_clone = fm_list.lock().unwrap().clone();
    let mut thread_handlers = Vec::new();

    for (file_idx, file) in files_clone.iter_mut().enumerate() {
        match file.ext.clone().as_str() {
            "pdf" => {
                thread_handlers.push(spawn(compress_pdf(
                    Arc::new(file.clone()),
                    file_idx,
                    fm_list.clone(),
                )));
            }
            _ => {}
        }
    }

    join_all(thread_handlers).await;
    fm_list
}
