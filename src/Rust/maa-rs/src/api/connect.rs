use std::net::SocketAddr;
use std::str::FromStr;

use crate::api::{Error, ManagerData};
use actix_web::web::{Json, Path};
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/connect/{id:\\d+}")
            .service(target)
            .service(attach),
    );
}

/// Returns the target of the specified MAA instance
#[post("/target")]
pub async fn target(id: Path<i64>, manager: ManagerData) -> Result<impl Responder, Error> {
    let id = id.into_inner();
    let manager = manager.lock().map_err(Error::from)?;

    manager.get(id).ok_or(Error::InstanceNotFound)?;
    let target = manager.target(id);

    Ok(HttpResponse::Ok().json(json!({"target": target})))
}

#[derive(Deserialize)]
pub struct Req {
    adb_path: String,
    target: String,
    config: Value,
}

/// Attaches to the MAA instance with the specified ID
///
/// # Parameters
/// * `id` - The ID of the MAA instance
#[post("/attach")]
pub async fn attach(
    id: Path<i64>,
    body: Json<Req>,
    manager: ManagerData,
) -> Result<impl Responder, Error> {
    let mut manager = manager.lock().map_err(Error::from)?;
    let id = id.into_inner();

    let maa = manager.get_mut(id).ok_or(Error::InstanceNotFound)?;

    let config = match body.config {
        Value::Null => None,
        _ => Some(body.config.to_string()),
    };

    let Ok(address) = SocketAddr::from_str(&body.target) else {
        return Err(Error::InvalidRequest);
    };

    maa.connect(&body.adb_path, address, config.as_ref())?;
    Ok(HttpResponse::Ok().finish())
}
