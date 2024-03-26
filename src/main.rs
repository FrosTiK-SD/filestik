extern crate google_drive3 as drive;

mod oauth;

use crate::routes::{download::download, shortcut::create_shortcut, upload::upload};
use actix_web::{middleware, web::Data, App, HttpServer};
use drive_manager::DriveManager;
use oauth::OAuthCredentialManager;
use tracing::{event, Level};
mod routes;

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
            .service(create_shortcut)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
    .expect("UNABLE TO START SERVER");

    Ok(())
}
