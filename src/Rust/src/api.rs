use crate::{
    database,
    maa_sys::{self, Maa},
};
use actix_web::{
    http::{header::ContentType, StatusCode},
    web, HttpResponse,
};
use serde_json::json;
use std::{collections::HashMap, ffi::c_void, sync::PoisonError};

mod connect;
mod device;
mod instances;
mod message;
mod run;
mod task;
mod uuid;
mod version;

/// Registers the API routes with the server
pub fn config(cfg: &mut web::ServiceConfig) {
    instances::config(cfg);
    connect::config(cfg);
    message::config(cfg);
    version::config(cfg);
    device::config(cfg);
    task::config(cfg);
    uuid::config(cfg);
    run::config(cfg);
}


#[derive(Debug)]
pub enum Error {
    Internal,
    InstanceNotFound,
    InvalidRequest,
}

impl From<database::Error> for Error {
    fn from(_: database::Error) -> Self {
        Self::Internal
    }
}
impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::Internal
    }
}

impl From<maa_sys::Error> for Error {
    fn from(_: maa_sys::Error) -> Self {
        Self::Internal
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl actix_web::error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let body = match self {
            Error::Internal => json!({
                "error":"Internal Error"
            }),
            Error::InstanceNotFound => json!({
                "error":"Instance Not Found"
            }),
            Error::InvalidRequest => json!({
                "error":"Invalid Request"
            }),
        };
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(body)
    }
    fn status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

pub type ManagerData = web::Data<std::sync::Mutex<MaaManager>>;

pub struct MaaManager {
    pub instances: HashMap<i64, Maa>,
    id: i64,
}

impl MaaManager {
    pub fn new() -> Self {
        MaaManager {
            instances: HashMap::new(),
            id: 0,
        }
    }
    pub fn create(&mut self) -> i64 {
        let id = self.gen_id();
        let maa = Maa::with_callback_and_custom_arg(
            Some(database::maa_store_callback),
            id as *mut c_void,
        );
        self.instances.insert(id, maa);
        id
    }
    pub fn get(&self, id: i64) -> Option<&Maa> {
        self.instances.get(&id)
    }
    pub fn get_mut(&mut self, id: i64) -> Option<&mut Maa> {
        self.instances.get_mut(&id)
    }
    pub fn get_target(&self, id: i64) -> Option<String> {
        let maa = self.get(id)?;
        maa.get_target()
    }
    pub fn delete(&mut self, id: i64) -> Option<Maa> {
        self.instances.remove(&id)
    }
    pub fn get_all_id(&self) -> Vec<i64> {
        self.instances.keys().copied().collect()
    }
    pub fn gen_id(&mut self) -> i64 {
        self.id += 1;
        self.id
    }
}

unsafe impl Send for MaaManager {}
