use std::any::Any;

use actix_web::web::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse<T> {
    success: bool,
    details: Option<T>,
    error: Option<String>,
    message: Option<String>,
}

impl<T: Any> GenericResponse<T> {
    pub fn ok(message: &str, details: Option<T>) -> Json<Self> {
        Json(Self {
            success: true,
            details: details,
            message: Some(message.to_string()),
            error: None,
        })
    }

    pub fn error(message: &str) -> Json<Self> {
        Json(Self {
            success: false,
            details: None,
            message: None,
            error: Some(message.to_string()),
        })
    }
}
