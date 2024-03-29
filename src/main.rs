#[macro_use] extern crate rocket;

mod auth;
mod schema;
mod models;

use diesel::prelude::*;
use auth::BasicAuth;
use rocket::serde::json::{Value, json, Json};
use rocket::response::status;
use rocket_sync_db_pools::database;
use schema::rustaceans;
use models::{Rustacean, NewRustacean};

#[database("sqlite")]

struct DbConn(diesel::SqliteConnection);


#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, db: DbConn) -> Value {
    db.run(|c| {
        let rustaceans = rustaceans::table.order(rustaceans::id.desc())
            .limit(1000)
            .load::<Rustacean>(c)
            .expect("DB error");
        json!(rustaceans)
    }).await
}
#[get("/rustaceans/<id>")]
async fn view_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> Value {
    db.run(move |c| {
        let rustacean = rustaceans::table.find(id)
            .get_result::<Rustacean>(c)
            .expect("DB error when selecting rustacean");
        json!(rustacean)
    }).await
}
#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustacean(_auth: BasicAuth, db: DbConn, new_rustacean: Json<NewRustacean>) -> Value {
    db.run(|c| {
        let result = diesel::insert_into(rustaceans::table)
            .values(new_rustacean.into_inner())
            .execute(c)
            .expect("DB error when inserting");
        json!(result)
    }).await
}
#[put("/rustaceans/<id>", format = "json", data = "<rustacean>")]
async fn update_rustacean(id: i32, _auth: BasicAuth, db: DbConn, rustacean: Json<Rustacean>) -> Value {
    db.run(move |c| {
        let result = diesel::update(rustaceans::table.find(id))
            .set((
                rustaceans::name.eq(rustacean.name.to_owned()),
                rustaceans::email.eq(rustacean.email.to_owned())
            ))
            .execute(c)
            .expect("DB error when updating");
        json!(result)
    }).await
}
#[delete("/rustaceans/<id>")]
async fn delete_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> status::NoContent {
    db.run(move |c| {
        diesel::delete(rustaceans::table.find(id))
            .execute(c)
            .expect("DB error when deleting");
        status::NoContent
    }).await
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
