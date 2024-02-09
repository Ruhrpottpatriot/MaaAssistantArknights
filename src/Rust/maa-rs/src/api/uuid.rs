use super::Error;
use crate::api::ManagerData;
use crate::database;
use actix_web::{
    get,
    web::{self, Path},
    HttpResponse, Responder,
};
use serde_json::json;

/// Registers all the `/uuid` routes
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/uuid").service(all).service(single));
}

/// Gets all UUIDs stored in the database
///
/// # Parameters
/// * None
///
/// # Returns
/// A list of all UUIDs stored in the database. Returns an empty array if there are no
/// UUIDs in the database.
#[get("")]
async fn all() -> Result<impl Responder, Error> {
    let uuids = database::get_all_uuid().map_err(Error::from)?;
    Ok(HttpResponse::Ok().json(uuids))
}

/// Gets the UUID of a specific MAA instance
///
/// # Parameters
/// * `id` - The ID of the MAA instance
///
/// # Returns
/// The UUID of the MAA instance, or a 404 not found error if the instance does not exist
#[get("/{id:\\d+}")]
pub async fn single(id: Path<i64>, manager: ManagerData) -> Result<impl Responder, Error> {
    let mut manager = manager.lock().map_err(Error::from)?;
    let id = id.into_inner();

    let maa = manager.get_mut(id).ok_or(Error::InstanceNotFound)?;
    let uuid = maa.uuid()?;

    Ok(HttpResponse::Ok().json(json!({
        "uuid": uuid
    })))
}
