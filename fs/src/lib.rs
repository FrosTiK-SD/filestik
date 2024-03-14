use std::fs;

use anyhow::{Ok, Result};
use google_drive3::{api::File, hyper::body::Bytes};

pub struct FileManager {
    pub file: File,
    pub base_path: String,
    pub file_name: String,
    pub mime_type: String,
    pub ext: String,
}

impl FileManager {
    pub fn new(file: File, base_path: String) -> Self {
        let (mime_type, ext) = Self::get_mime_type_and_ext(file.clone());

        Self {
            file: file.clone(),
            base_path,
            file_name: Self::get_file_name(file, ext.clone()),
            mime_type,
            ext,
        }
    }

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

    pub fn get_target_path(&self) -> String {
        format!("{}/{}", self.base_path, self.file_name)
    }

    pub async fn write_file(&self, content: Bytes) -> Result<()> {
        fs::create_dir_all(self.base_path.as_str()).unwrap();
        fs::write(self.get_target_path(), &content).unwrap();
        Ok(())
    }
}
