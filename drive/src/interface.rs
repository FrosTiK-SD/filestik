use std::fs::File;

use mime_guess::Mime;

pub struct CreateFileStruct {
    pub file_path: String,
    pub name: String,
    pub ext: Option<String>,
    pub mime_type: Option<Mime>,
    pub parents: Vec<String>,
    pub content: File,
}
