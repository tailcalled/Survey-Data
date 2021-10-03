pub mod tests;
pub mod routes;
pub mod database;

#[macro_use] extern crate rocket;

use rocket_dyn_templates::Template;
use sass_rocket_fairing::SassFairing;
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
						routes::test::post_test,
						routes::test::test,
						routes::test::post_feedback,
						routes::test::get_feedback,
						routes::debug::all_responses,
						routes::statics::style])
                    .manage::<PgPool>(pool)
                    .manage(make_tests())
					.attach(SassFairing)
                    .attach(Template::custom( |engines| {
                        routes::customize(&mut engines.tera);
                    }))

}
