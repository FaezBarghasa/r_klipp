use actix_web::{web, App, HttpServer, Responder};
use anyhow::Result;
use env_logger::Env;
use log::info;

mod db;
mod api;
mod bridge;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    info!("Starting r_klipp host server...");

    // Initialize SurrealDB
    let db = db::Database::new().await?;
    db.init_schema().await?;
    info!("SurrealDB initialized and schema applied.");

    // Start Actix-Web server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone())) // Pass database to handlers
            .service(web::resource("/").to(|| async { "r_klipp Host Server" }))
            // TODO: Add API and WebSocket services here
    })
    .bind("0.0.0.0:7125")?
    .run()
    .await?;

    info!("r_klipp host server stopped.");
    Ok(())
}