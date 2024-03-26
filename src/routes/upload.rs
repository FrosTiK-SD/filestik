use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{
    post,
    web::{Data, Json},
    HttpRequest, Responder, Result,
};
use drive_manager::{interface::CreateFileStruct, link::Link, DriveManager};
use serde::Serialize;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "parent")]
    parents: Vec<Text<String>>,
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
    #[multipart(rename = "link")]
    links: Vec<Text<String>>,
}

#[derive(Serialize)]
struct UploadResponse {
    message: &'static str,
    urls: Vec<Option<String>>,
}

#[post("/upload")]
pub async fn upload(
    req: HttpRequest,
    drive_manager: Data<DriveManager>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder> {
    let mut file_paths = vec![];
    let mut parents = vec![];

    for parent in form.parents {
        parents.push(parent.to_string());
    }

    for (idx, file) in form.files.iter().enumerate() {
        let content = file.file.as_file().try_clone().unwrap();
        let file_path = file.file_name.clone().unwrap_or_default();
        let file_path_clone = file_path.clone();
        let file_path_parts: Vec<&str> = file_path_clone.split(".").collect();
        let ext = if file_path_parts.len() > 1 {
            Some(file_path_parts[file_path_parts.len() - 1].to_string())
        } else {
            None
        };
        let file_id = if form.links.get(idx).is_some() {
            Some(Link::new(form.links.get(idx).clone().unwrap().to_string()).id)
        } else {
            None
        };

        file_paths.push(CreateFileStruct {
            file_path,
            name: file.file_name.clone().unwrap(),
            mime_type: file.content_type.clone(),
            ext,
            file_id,
            parents: parents.clone(),
            content,
        });
    }

    let urls = drive_manager.upload_files(file_paths).await.unwrap();

    Ok(Json(UploadResponse {
        message: "Data Uploaded Successfully",
        urls,
    }))
}
