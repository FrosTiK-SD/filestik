use actix_files as afs;
use actix_files::file_extension_to_mime;
use actix_web::{
    get,
    http::header::{ContentDisposition, ContentEncoding},
    web::Data,
    HttpRequest, HttpResponse, Result,
};
use drive::{chrono::Utc, hyper::StatusCode};
use drive_manager::DriveManager;

#[get("/download")]
pub async fn download(req: HttpRequest, drive_manager: Data<DriveManager>) -> Result<HttpResponse> {
    let start_time = Utc::now().time();
    let link = req.headers().get("link");

    if link.is_none() {
        return Ok(HttpResponse::new(StatusCode::BAD_REQUEST));
    }

    drive_manager
        .download_file(link.unwrap().to_str().unwrap())
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
