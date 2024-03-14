use std::{
    fs,
    os::unix::process::CommandExt,
    sync::{Arc, Mutex},
};

use pdfshrink::gs_command;

use crate::FileManager;

pub async fn compress(files: Arc<Mutex<Vec<FileManager>>>) -> Arc<Mutex<Vec<FileManager>>> {
    let mut files_clone = files.lock().unwrap().clone();

    for (file_idx, file) in files_clone.iter_mut().enumerate() {
        match file.ext.clone().as_str() {
            "pdf" => {
                let output_path = file.get_compressed_target_path();
                let error = gs_command(file.get_optimal_target_path(), output_path.clone()).spawn();
                if error.is_ok() {
                    files
                        .lock()
                        .unwrap()
                        .get_mut(file_idx)
                        .unwrap()
                        .compressed_file_path = output_path;
                }
            }
            _ => {}
        }
    }
    files
}
