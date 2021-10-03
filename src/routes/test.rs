use rocket::serde::Serialize;
use rocket::form::Form;
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use rocket::State;
use rocket::http::{CookieJar, Cookie};
use rocket::response::Redirect;
use uuid::Uuid;
use crate::tests::*;
use super::{TemplateContext, DebugContext, style_hash};

pub static TEST_TEMPLATE: &str = r#"
{% extends "base" %}

{% block content %}
	<form action="/feedback/{{data.test.id}}" method="post">
		{% for element in data.test.elements %}
		    {% if element.content.McQuestion %}
		        {% set content = element.content.McQuestion %}
                <div class="question">
                    <div class="question-text">{{ content.text }}</div>
                    <div class="question-options">
                        {% for opt in content.options %}
                            <div class="question-option">
                                <input type="radio" id="{{ element.id }}_{{ loop.index-1 }}" name="questions.{{ element.id }}" value="{{ loop.index-1 }}">
                                <label for="{{ element.id }}_{{ loop.index-1 }}">{{ opt }}</label>
                            </div>
                        {% endfor %}
                    </div>
                </div>
            {% endif %}
		{% endfor %}
		<input class="submit" type="submit" value="Get Results!">
	</form>
{% endblock content %}
"#;

pub static FEEDBACK_TEMPLATE: &str = r#"
{% extends "base" %}

{% block content %}
    {% for element in data.feedback %}
        {% if element.Title %}
            {% set content = element.Title %}
            <h2>{{content.text}}</h2>
        {% elif element.Paragraph %}
            {% set content = element.Paragraph %}
            <p>{{content.text}}</p>
        {% elif element.Bar %}
            {% set content = element.Bar %}
            {% set percentage = 100.0 * (content.score - content.min) / (content.max - content.min) %}
            <div class="bar-container">
                <div class="bar-percent">{{percentage | round}}%</div>
                <div class="bar">
                    <div class="bar-fill" style="width: {{percentage}}%"></div>
                    <div class="bar-empty" style="width: {{100-percentage}}%"></div>
                </div>
            </div>
        {% endif %}
    {% endfor %}
{% endblock content %}
"#;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TestContext<'r> {
    test: &'r Test,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct FeedbackContext<'r> {
    feedback: &'r Vec<FeedbackItem>,
}

#[derive(FromForm)]
#[derive(Debug)]
pub struct Response {
    questions: HashMap<String, String>
}

#[get("/test/<test>")]
pub async fn test(test: &Test, cookies: &CookieJar<'_>) -> Option<Template> {
    if cookies.get("responseId").is_none() {
        cookies.add(Cookie::new("responseId", Uuid::new_v4().to_string()));
    }
    Some(Template::render("test.html", &TemplateContext {
        title: &test.name,
        style_hash: &style_hash().await?,
        data: TestContext { test: test },
    }))
}

#[post("/feedback/<test>", data="<response>")]
pub async fn post_feedback(test: &Test, cookies: &CookieJar<'_>, response: Form<Response>, pool: &State<PgPool>) -> Redirect {
    let response_id: Uuid = cookies.get("responseId").unwrap().value().parse().unwrap();
    let mut resp_map = HashMap::new();
    for question in &test.elements {
        let it = question.content.convert(&response.questions[&question.id]);
        resp_map.insert(question.id.clone(), it);
    }
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query!(
		"INSERT INTO responses(response_id, user_id, submit_time, content)
		               VALUES ($1, $2, NOW(), $3) ON CONFLICT DO NOTHING",
		response_id, Option::<Uuid>::None, serde_json::to_value(&resp_map).unwrap()
	).execute(&mut conn).await.unwrap();
    cookies.remove(Cookie::named("responseId"));
    Redirect::to(uri!(get_feedback(test=test, id=response_id.to_string())))
}
#[get("/feedback/<test>/<id>")]
pub async fn get_feedback(test: &Test, pool: &State<PgPool>, id: &str) -> Option<Template> {
    let response_id: Uuid = id.parse().unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let res = sqlx::query!(
		"SELECT response_id, user_id, submit_time, content FROM responses WHERE response_id = $1",
		response_id
	).fetch_one(&mut conn).await.unwrap();
    let mut feedback = vec![];
    for part in &test.feedback {
        feedback.push(part.score(&res.content));
    }
    Some(Template::render("feedback.html", &TemplateContext {
        title: "Feedback",
        style_hash: &style_hash().await?,
        data: FeedbackContext {
            feedback: &feedback
        }
    }))
//    Some(Template::render("debug.html", &TemplateContext {
//        title: "Feedback",
//        style_hash: &style_hash().await?,
//        data: DebugContext { body: &format!("{:#?}", res) }
//    }))
}