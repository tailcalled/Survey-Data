use rocket_dyn_templates::{Template};
use super::{TemplateContext, style_hash};

pub static TEMPLATE: &str = r#"
{% extends "base" %}

{% block content %}
    <h1>Hello, world!</h1>
    <p>This is a website that hosts psychological tests, especially self-report personality tests.
    Consider taking one of the tests below to get feedback on yourself:</p>
    <ul>
        <li> <a href="/test/tipi/0">Ten-Item Personality Inventory</a>: an ultra-quick personality
        test developed to be quickly able to guesstimate someone's personality.
    </ul>
    <p>The website is still under construction, so I apologize if it is a bit rough around the
    edges.</p>
{% endblock content %}
"#;

#[get("/")]
pub async fn index() -> Template {
    Template::render("index.html", &TemplateContext {
        title: "Index",
        style_hash: &style_hash().await,
        data: (),
    })
}
