use super::Error;
use actix_web::{get, web, HttpResponse, Responder};
use maa_rs_sys::{Assistant,get_version};
use serde::Serialize;
use serde_json::json;

/// Registers the `/version` route
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(version);
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
///     "server": {
///         "full": "0.1.0",
///         "major": 0,
///         "minor": 1,
///         "patch": 0
///     }
/// }
/// ```
#[get("/version")]
pub async fn version() -> Result<impl Responder, Error> {
    fn is_none_or_empty(value: &Option<&str>) -> bool {
        match value {
            Some(value) => value.is_empty(),
            None => true,
        }
    }

    // Local struct that holds the server version. Defined to easily control
    // that `None` values are not serialized.
    #[derive(Debug, Serialize)]
    #[serde(rename = "server")]
    struct VersionSpec<'a> {
        full: &'a str,
        major: &'a str,
        minor: &'a str,
        patch: &'a str,

        #[serde(skip_serializing_if = "is_none_or_empty")]
        build: Option<&'a str>,

        #[serde(skip_serializing_if = "is_none_or_empty")]
        pre: Option<&'a str>,
    }

    let server = VersionSpec {
        full: env!("CARGO_PKG_VERSION"),
        major: env!("CARGO_PKG_VERSION_MAJOR"),
        minor: env!("CARGO_PKG_VERSION_MINOR"),
        patch: env!("CARGO_PKG_VERSION_PATCH"),
        build: option_env!("CARGO_PKG_VERSION_BUILD"),
        pre: option_env!("CARGO_PKG_VERSION_PRE"),
    };
    // TODO: Get the version for the FFI crate

    let core = get_version()?;

    Ok(HttpResponse::Ok().json(json!({
        "core": core,
        "server": server,
    })))
}
