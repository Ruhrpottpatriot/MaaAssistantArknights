use std::sync::Mutex;

use actix_web::{
    post, web,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;
use serde_json::json;

use crate::api::{Error, MaaManager, ManagerData};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("device/{id:\\d+}")
            .service(click)
            .service(screenshot),
    );
}

#[derive(Deserialize)]
pub struct Coord {
    x: i32,
    y: i32,
}

/// Clicks on the screen at the specified coordinates
///
/// # Parameters
/// * `id` - The ID of the MAA instance
#[post("/click")]
pub async fn click(
    id: Path<i64>,
    req: Json<Coord>,
    manager: ManagerData,
) -> Result<impl Responder, Error> {
    let manager = manager.lock().map_err(Error::from)?;

    // TODO: Transform the click result into a proper response
    let _return_code = manager
        .get(id.into_inner())
        .ok_or(Error::InstanceNotFound)?
        .click(req.x, req.y)?;

    Ok(HttpResponse::Ok().finish())
}

/// Creates a screenshot for the specified MAA instance
#[post("/screenshot")]
pub async fn screenshot(id: Path<i64>, manager: ManagerData) -> Result<impl Responder, Error> {
    let manager = manager.lock().map_err(Error::from)?;

    let body = manager
        .get(id.into_inner())
        .ok_or(Error::InstanceNotFound)?
        .screenshot()?;

    Ok(HttpResponse::Ok().body(body))
}
