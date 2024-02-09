use actix_web::web;
mod get;
mod drop;
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/message/get", web::get().to(get::get));
    cfg.route("/message/drop", web::post().to(drop::drop));
}
