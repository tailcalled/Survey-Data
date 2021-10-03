pub mod tests;
pub mod routes;

#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket_dyn_templates::Template;
use sqlx::postgres::{PgPool, PgPoolOptions};
use crate::tests::make_tests;

#[launch]
async fn rocket() -> _ {
    // CREATE USER surveydata WITH PASSWORD 'surveydata' CREATEDB;
    // CREATE DATABASE surveydata;
    let database_url = "postgres://surveydata:surveydata@localhost/surveydata";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url).await.unwrap();
    rocket::build().mount("/", routes![
						routes::index::index,
						routes::test::test,
						routes::test::post_feedback,
						routes::test::get_feedback,
						routes::statics::style])
                    .manage::<PgPool>(pool)
                    .manage(make_tests())
                    .attach(Template::custom( |engines| {
                        routes::customize(&mut engines.tera);
                    }))

}
