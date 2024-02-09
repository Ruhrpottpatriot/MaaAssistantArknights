use crate::api::ManagerData;

use super::{Error, MaaManager};
use actix_web::{
    post,
    web::{self, Json, Path},
    HttpResponse, Responder,
};
use std::sync::Mutex;

/// Registers the endpoints for the run API
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/run/{id:\\d+}").service(start).service(stop));
}

/// Starts the MAA instance with the given ID
///
/// # Parameters
/// * `id` - The ID of the MAA instance
#[post("/start")]
pub async fn start(id: Path<i64>, manager: ManagerData) -> Result<impl Responder, Error> {
    let manager = manager.lock().map_err(Error::from)?;
    let id = id.into_inner();

    let maa = manager.get(id).ok_or(Error::InstanceNotFound)?;
    maa.start()?;

    Ok(HttpResponse::Ok().finish())
}

/// Stops the MAA instance with the given ID
///
/// # Parameters
/// * `id` - The ID of the MAA instance
#[post("/stop")]
pub async fn stop(id: Json<i64>, manager: ManagerData) -> Result<impl Responder, Error> {
    let manager = manager.lock().map_err(Error::from)?;
    let id = id.into_inner();

    let maa = manager.get(id).ok_or(Error::InstanceNotFound)?;
    maa.stop()?;

    Ok(HttpResponse::Ok().finish())
}
