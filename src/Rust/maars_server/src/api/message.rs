use crate::database;

use super::Error;
use actix_web::{
    delete, get,
    web::{self, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;
use serde_json::{json, Value};

/// Registers the endpoints for the message API
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("mesage").service(messages).service(drop));
}
#[derive(Deserialize)]
pub struct Req {
    nums: Option<i64>,
}

/// Returns the last `n` messages for the given device from database
#[get("{uuid}")]
pub async fn messages(id: Path<String>, req: Json<Req>) -> Result<impl Responder, Error> {
    let id = id.into_inner();
    let nums = req.nums.unwrap_or(i64::MAX);

    let msgs = database::get_last_msg(&id, nums as usize).map_err(Error::from)?;

    let mut ret: Vec<Value> = Vec::new();
    for x in msgs {
        let body: Value = serde_json::from_str(&x.body).map_err(|_| Error::Internal)?;
        let json = json!({
            "time": x.time,
            "body": body,
            "type": x.type_,
        });
        ret.push(json)
    }
    let ret = json!(ret);

    Ok(HttpResponse::Ok().json(ret))
}

/// Deletes all messages for the given deviceid
///
/// # Parameter
/// *`id` - The id of the device
#[delete("{uuid}")]
pub async fn drop(id: Path<String>) -> Result<impl Responder, Error> {
    let uuid = id.into_inner();

    database::drop(&uuid).map_err(|_| Error::Internal)?;
    Ok(HttpResponse::Ok().finish())
}
