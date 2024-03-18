extern crate google_drive3 as drive;

mod oauth;
use actix_files as afs;
use afs::file_extension_to_mime;
use std::{any::Any, os::fd::AsFd};

use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        text::Text,
        MultipartForm,
    },
    Multipart,
};
use actix_web::{
    get,
    http::header::{ContentDisposition, ContentEncoding, ContentType},
    middleware, post,
    web::{Data, Json},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use drive::chrono::Utc;
use drive_manager::{interface::CreateFileStruct, DriveManager};
use oauth::OAuthCredentialManager;
use serde::Serialize;
use tracing::{event, Level};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "parent")]
    parents: Vec<Text<String>>,
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[derive(Serialize)]
struct UploadResponse {
    message: &'static str,
    urls: Vec<Option<String>>,
}

#[get("/download")]
async fn download(req: HttpRequest, drive_manager: Data<DriveManager>) -> Result<HttpResponse> {
    let start_time = Utc::now().time();

    drive_manager
        .download_file("https://drive.google.com/drive/folders/1LeKGGNtD8ZOmKi2bfRh0uJltS5B-wq-z")
        .await
        .unwrap();

    let end_time = Utc::now().time();
    let diff = end_time - start_time;

    println!(
        "--FINISHED_DOWNLOAD-- in {:?} secs",
        diff.num_milliseconds()
    );

    let file = afs::NamedFile::open("tmp/output.zip").unwrap();
    Ok(file
        .set_content_type(file_extension_to_mime("zip"))
        .set_content_encoding(ContentEncoding::Gzip)
        .set_content_disposition(ContentDisposition::attachment("output.zip"))
        .into_response(&req))
}

#[post("/upload")]
async fn upload(
    req: HttpRequest,
    drive_manager: Data<DriveManager>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> Result<impl Responder> {
    let mut file_paths = vec![];
    let mut parents = vec![];

    for parent in form.parents {
        parents.push(parent.to_string());
    }

    for file in form.files {
        let content = file.file.as_file().try_clone().unwrap();
        let file_path = file.file_name.clone().unwrap_or_default();
        let file_path_clone = file_path.clone();
        let file_path_parts: Vec<&str> = file_path_clone.split(".").collect();
        let ext = if file_path_parts.len() > 1 {
            Some(file_path_parts[file_path_parts.len() - 1].to_string())
        } else {
            None
        };

        file_paths.push(CreateFileStruct {
            file_path,
            name: file.file_name.clone().unwrap(),
            mime_type: file.content_type.clone(),
            ext,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let cred_manager = OAuthCredentialManager::default_initialize().await.unwrap();
    let drive_manager =
        DriveManager::new(cred_manager.connector.unwrap()).expect("Cant initialize drive manager");

    event!(
        Level::INFO,
        "TRYING TO START SERVER AT http://localhost:8080"
    );

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .app_data(Data::new(drive_manager.clone()))
            .service(download)
            .service(upload)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
    .expect("UNABLE TO START SERVER");

    Ok(())
}
