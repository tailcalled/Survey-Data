use rocket::State;
use rocket_dyn_templates::Template;
use sqlx::PgPool;
use crate::database;
use crate::routes::{DebugContext, style_hash, TemplateContext};

#[get("/debug/all_responses")]
pub async fn all_responses(pool: &State<PgPool>) -> Template {
    let res = database::get_all_responses(&mut pool.acquire().await.unwrap()).await;
    Template::render("debug.html", &TemplateContext {
        title: "Debug - All Responses",
        style_hash: &style_hash().await,
        data: DebugContext { body: &res }
    })
}