#[macro_use] extern crate rocket;

mod auth;
mod schema;
mod models;
mod repositories;

use auth::BasicAuth;
use diesel::result::Error::NotFound;
use rocket::http::Status;
use rocket::serde::json::{Value, json, Json};
use rocket::response::status::{self, Custom};
use rocket_sync_db_pools::database;
use models::{Rustacean, NewRustacean};
use repositories::RustaceanRepository;

#[database("sqlite")]

struct DbConn(diesel::SqliteConnection);


#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(|c| {
        RustaceanRepository::find_multiple(c, 100)
            .map(|rustaceans| json!(rustaceans))
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[get("/rustaceans/<id>")]
async fn view_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::find(c, id)
            .map(|rustacean| json!(rustacean))
            .map_err(|e|
                match e {
                    NotFound => Custom(Status::NotFound, json!(e.to_string())),
                    _ => Custom(Status::InternalServerError, json!(e.to_string()))
                }
            )
    }).await
}
#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustacean(_auth: BasicAuth, db: DbConn, new_rustacean: Json<NewRustacean>) -> Result<Value, Custom<Value>> {
    db.run(|c| {
        RustaceanRepository::create(c, new_rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[put("/rustaceans/<id>", format = "json", data = "<rustacean>")]
async fn update_rustacean(id: i32, _auth: BasicAuth, db: DbConn, rustacean: Json<Rustacean>) -> Result<Value, Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::save(c, id, rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
    }).await
}
#[delete("/rustaceans/<id>")]
async fn delete_rustacean(
    id: i32,
    _auth: BasicAuth,
    db: DbConn,
) -> Result<status::NoContent, Custom<Value>> {
    db.run(move |connection| {
        match RustaceanRepository::find(connection, id) {
            Ok(_) => RustaceanRepository::delete(connection, id)
                .map(|_| status::NoContent)
                .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string()))),
            Err(e) => Err(Custom(Status::NotFound, json!(e.to_string()))),
        }
    })
    .await
}

#[catch(404)]
fn not_found() -> Value {
    json!("Not found!")
}

#[catch(401)]
fn unauthorized() -> Value {
    json!("Unauthorized!")
}

#[catch(422)]
fn unprocessable() -> Value {
    json!("Invalid entity. Probably some missing fields?")
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
    .mount("/", routes![
        get_rustaceans,
        view_rustacean,
        create_rustacean,
        update_rustacean,
        delete_rustacean
    ])
    .register("/", catchers![
        not_found,
        unauthorized,
        unprocessable
    ])
    .attach(DbConn::fairing())
    .launch()
    .await;
}
