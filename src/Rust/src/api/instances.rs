use crate::api::ManagerData;

use super::{Error};
use actix_web::web::{self, Json};
use actix_web::{delete, get, post, HttpResponse, Responder};
use serde_json::json;


/// Registers all `/instances` routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/instances")
            .service(instances)
            .service(create)
            .service(delete),
    );
}

/// Gets all instances stored in the database
#[get("")]
pub async fn instances(manager: ManagerData) -> Result<impl Responder, Error> {
    let instances = manager
        .lock()
        .map_err(Error::from)?
        .get_all_id();

    Ok(HttpResponse::Ok().json(json!(instances)))
}

/// Creates a new MAA instance
#[post("")]
pub async fn create(manager: ManagerData)-> Result<impl Responder, Error>{
    let id = manager.lock().map_err(|_|Error::Internal)?.create();

    Ok(HttpResponse::Ok().json(json!({"id": id})))
}

/// Deletes a MAA instance
/// 
/// # Parameters
/// * `id` - The ID of the MAA instance
#[delete("{id:\\d+}")]
pub async fn delete(req: Json<i64>, manager: ManagerData)-> Result<impl Responder, Error>{
    manager.lock()
    .map_err(|_|Error::Internal)?
    .delete(req.into_inner())
    .ok_or(Error::InstanceNotFound)?;

    Ok(HttpResponse::Ok().finish())
}
