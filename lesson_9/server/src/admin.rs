#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::Request;
use rocket_db_pools::{sqlx, Connection, Database};
use rocket_dyn_templates::{context, Template};

#[derive(Database)]
#[database("server_db")]
struct Server(sqlx::SqlitePool);

#[derive(FromForm)]
struct Query {
    nickname: String,
}

#[get("/")]
async fn index() -> Template {
    Template::render("index", context! {title: "Admin"})
}

#[get("/")]
async fn messages(mut db: Connection<Server>) -> Template {
    let rows: Vec<(i64, String, String, String)> = sqlx::query_as("SELECT * FROM messages;")
        .fetch_all(&mut **db)
        .await
        .unwrap_or(Vec::new());
    Template::render("messages", context! {title: "Messages", rows: rows})
}

#[get("/form")]
async fn messages_form() -> Template {
    Template::render("messages_form", context! {title: "Messages Form"})
}

#[post("/nickname", data = "<query_form>")]
async fn messages_nickname(mut db: Connection<Server>, query_form: Form<Query>) -> Template {
    let nickname = &query_form.nickname;
    let rows: Vec<(i64, String, String, String)> =
        sqlx::query_as("SELECT * FROM messages WHERE nickname = ( ?1 );")
            .bind(nickname)
            .fetch_all(&mut **db)
            .await
            .unwrap_or(Vec::new());
    Template::render("messages", context! {title: "Messages", rows: rows})
}

#[get("/form")]
async fn delete_form() -> Template {
    Template::render("delete_form", context! {title: "Delete Form"})
}

#[post("/nickname", data = "<query_form>")]
async fn delete_nickname(mut db: Connection<Server>, query_form: Form<Query>) -> Template {
    let nickname = &query_form.nickname;
    let rows = match sqlx::query("DELETE FROM messages WHERE nickname = ( ?1 );")
        .bind(nickname)
        .execute(&mut **db)
        .await
    {
        Ok(result) => result.rows_affected(),
        Err(_) => 0,
    };

    Template::render("delete", context! {title: "Delete", rows: rows})
}

#[catch(404)]
async fn not_found(request: &Request<'_>) -> Template {
    Template::render(
        "404",
        context! {
            uri: request.uri()
        },
    )
}

#[launch]
async fn rocket() -> _ {
    rocket::build()
        .attach(Server::init())
        .mount("/", routes![index])
        .mount(
            "/messages",
            routes![messages, messages_form, messages_nickname],
        )
        .mount("/delete", routes![delete_form, delete_nickname])
        .register("/", catchers![not_found])
        .attach(Template::fairing())
}
