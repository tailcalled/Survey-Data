use rocket::serde::Serialize;
use rocket_dyn_templates::tera::Tera;
use tokio::fs::metadata;

pub mod index;
pub mod statics;
pub mod test;
pub mod debug;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TemplateContext<'r, A> {
    title: &'r str,
    style_hash: &'r str,
    data: A,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DebugContext<'r> {
    body: &'r str,
}

pub async fn style_hash() -> Option<String> {
    Some(format!("{:?}", metadata("static/style.css").await.ok()?.modified().ok()?))
}

static DEBUG_TEMPLATE: &str = r#"
{% extends "base" %}

{% block content %}
    <h1>Debug Page</h1>
    <pre>{{ data.body }}</pre>
{% endblock content %}
"#;

pub fn customize(tera: &mut Tera) {
    tera.add_raw_template("test.html", test::TEST_TEMPLATE).unwrap();
    tera.add_raw_template("feedback.html", test::FEEDBACK_TEMPLATE).unwrap();
    tera.add_raw_template("index.html", index::TEMPLATE).unwrap();
    tera.add_raw_template("debug.html", DEBUG_TEMPLATE).unwrap();
}