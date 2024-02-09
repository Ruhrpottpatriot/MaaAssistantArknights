use maa_rs_sys::Maa;
use crate::SERVER_VERSION;
use super::Error;
use actix_web::{HttpResponse, Responder, web};
use serde_json::json;

/// Registers the `/version` route
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/version", web::get().to(version));
}

/// Gets the version of the MAA core library and the server
/// 
/// # Parameters
/// * None
/// 
/// # Returns
/// A JSON object containing the version of the MAA core library and the server, e.g.
/// ```json
/// {
///     "core": "v0.0.1",
///     "server": "v0.1.0"
/// } 
/// ```
pub async fn version() -> Result<impl Responder, Error> {
    let core_version = Maa::get_version()?;
    Ok(HttpResponse::Ok().json(json!({
        "core":core_version,
        "server":SERVER_VERSION
    })))
}
