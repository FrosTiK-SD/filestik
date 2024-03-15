extern crate google_drive3 as drive;

mod oauth;
use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use drive::chrono::Utc;
use drive_manager::DriveManager;
use oauth::OAuthCredentialManager;

#[get("/")]
async fn hello(drive_manager: Data<DriveManager>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cred_manager = OAuthCredentialManager::default_initialize().await.unwrap();
    let drive_manager =
        DriveManager::new(cred_manager.connector.unwrap()).expect("Cant initialize drive manager");

    let start_time = Utc::now().time();

    drive_manager
        .download_file("https://drive.google.com/drive/folders/1LeKGGNtD8ZOmKi2bfRh0uJltS5B-wq-z")
        .await
        .unwrap();
    // drive_manager
    // .download_file(
    //     "https://drive.google.com/drive/folders/1sVqdKiRPsET4RGBhUYv9S7pmpzYGNBMo?usp=drive_link",
    // )
    // .await.unwrap();

    let end_time = Utc::now().time();
    let diff = end_time - start_time;

    println!("--FINISHED_DOWNLOAD-- in {:?} secs", diff.num_seconds());

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(drive_manager.clone()))
            .service(hello)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
