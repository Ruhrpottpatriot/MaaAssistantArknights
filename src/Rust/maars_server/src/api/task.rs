use crate::api::ManagerData;

use super::{Error, MaaManager};
use actix_web::{
    get, post, put,
    web::{self, Json, Path},
    HttpResponse, Responder,
};
use maars::TaskId;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Mutex};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tasks")
            .service(all)
            .service(set)
            .service(create),
    );
}

/// Gets all tasks of a specific MAA instance
///
/// # Parameters
/// * `id` - The ID of the MAA instance
///
/// # Returns
/// An array of tasks, or an error if the instance does not exist
#[get("/{id:\\d+}")]
pub async fn all(id: Path<i64>, manager: ManagerData) -> Result<impl Responder, Error> {
    let mut manager = manager.lock().map_err(Error::from)?;
    let id = id.into_inner();

    let maa = manager.get_mut(id).ok_or(Error::InstanceNotFound)?;
    let tasks = {
        let mut tmp = HashMap::new();
        for (k, v) in maa.tasks()? {
            tmp.insert(
                k.to_string(),
                json!({
                    "type": v.task_type,
                    "params": v.params
                }),
            );
        }

        tmp
    };

    Ok(HttpResponse::Ok().json(json!(tasks)))
}

#[derive(Deserialize)]
pub struct TaskUpdate {
    task_id: i32,
    params: Value,
}

/// Sets the parameters of a specific task of a specific MAA instance
///
/// This method accepts a JSON object in the body of the request, with the following fields:
/// * `task_id` - The ID of the task to set the parameters for
/// * `params` - The parameters to set for the task
///
/// # Parameters
/// * `id` - The ID of the MAA instance
#[put("/{id:\\d-}")]
pub async fn set(
    id: Path<i64>,
    body: Json<TaskUpdate>,
    manager: ManagerData,
) -> Result<impl Responder, Error> {
    let manager = manager.lock().map_err(Error::from)?;
    let id = id.into_inner();

    let maa = manager.get(id).ok_or(Error::InstanceNotFound)?;
    let params = match body.params {
        Value::Null => "{}".to_string(),
        Value::Object(_) => body.params.to_string(),
        _ => return Err(Error::InvalidRequest),
    };

    let task_id = TaskId::new(body.task_id);
    maa.set_task_parameters(task_id, &params)?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct Task {
    types: String,
    params: Value,
}

/// Creates a new task for a specific MAA instance
///
/// This method accepts a JSON object in the body of the request, with the following fields:
/// * `task_id` - The ID of the task to set the parameters for
/// * `params` - The parameters to set for the task
///
/// # Parameters
/// * `id` - The ID of the MAA instance
#[post("/{id:\\d+}")]
pub async fn create(
    id: Path<i64>,
    body: Json<Task>,
    maa_manager: web::Data<Mutex<MaaManager>>,
) -> Result<impl Responder, Error> {
    let mut manager = maa_manager.lock().map_err(|_| Error::Internal)?;
    let id = id.into_inner();

    let maa = manager.get_mut(id).ok_or(Error::InstanceNotFound)?;
    let params = match body.params {
        Value::Null => "{}".to_string(),
        Value::Object(_) => body.params.to_string(),
        _ => return Err(Error::InvalidRequest),
    };
    let task_id = maa.create_task(&body.types, &params)?;

    Ok(HttpResponse::Created().json(json!({"task_id": task_id})))
}
