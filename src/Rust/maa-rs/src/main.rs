#![allow(dead_code)]

mod api;
mod config;
mod database;

use actix_web::{middleware, rt, web, App, HttpServer};
use config::CONFIG;
use maa_rs_sys::Maa;
use std::{error::Error, sync::Mutex};



fn main() -> Result<(), Box<dyn Error>> {
    Maa::load_resource(&CONFIG.resource.path).unwrap();

    if CONFIG.database.drop_on_start_up {
        database::drop_all().unwrap();
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let maa_manager = web::Data::new(Mutex::new(api::MaaManager::new()));

    rt::System::new()
        .block_on(async {
            HttpServer::new(move || {
                App::new()
                    .app_data(maa_manager.clone())
                    .wrap(middleware::Logger::default())
                    .configure(api::config)
            })
            .bind((CONFIG.server.address, CONFIG.server.port))?
            .run()
            .await
        })
        .map_err(std::io::Error::into)
}
