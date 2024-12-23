use rocket::{get, options, post, routes, serde::json::Json};
use dotenv::dotenv;
use serde_json::json;

mod routes;
mod models;
mod db;
mod schema;
mod cors;

use routes::obsidian::update_or_create_obsidian_file_by_name;
use models::obsidian::{ObsidianFile, NewObsidianFile, create_obsidian_file_in_db, get_obsidian_file_by_id};
use cors::CORS;


#[options("/<_..>")]
fn options() -> rocket::http::Status {
    rocket::http::Status::Ok
}

#[post("/obsidian_file", data = "<obsidian_file>")]
async fn create_obsidian_file(obsidian_file:Json<NewObsidianFile>) -> Json<ObsidianFile> {
    let pool = db::get_connection_pool();
    let mut conn = pool.get().expect("Could not get connection from pool");
    let result = create_obsidian_file_in_db(&mut conn, obsidian_file.into_inner());
    Json(result)
}


#[post("/update_obsidian_file/<file_name>", data = "<new_content>")]
async fn update_obsidian_file(file_name: String, new_content: String) -> Result<Json<ObsidianFile>, Json<serde_json::Value>> {
    let pool = db::get_connection_pool();
    let mut conn = pool.get().expect("Could not get connection from pool");
    let result = update_or_create_obsidian_file_by_name(&mut conn, file_name, new_content).await;
    match result {
        Some(file) => Ok(Json(file)),
        None => Err(Json(json!({"error": "not found"}))),
    }
}


#[get("/obsidian_file/<file_id>")]
async fn get_obsidian_file(file_id: i32) -> Result<Json<ObsidianFile>, Json<serde_json::Value>> {
    let pool = db::get_connection_pool();
    let mut conn = pool.get().expect("Could not get connection from pool");
    let result = get_obsidian_file_by_id(&mut conn, file_id);
    match result {
        Some(file) => Ok(Json(file)),
        None => Err(Json(json!({"error": "not found"}))),
    }
}



#[rocket::main]
async fn main() {
    dotenv().ok();

    rocket::build()
        .attach(CORS)
        .mount("/", routes![create_obsidian_file, get_obsidian_file, update_obsidian_file, options])
        .launch()
        .await
        .expect("server failed to launch");

}