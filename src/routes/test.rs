use rocket::serde::Serialize;
use rocket::form::Form;
use rocket_dyn_templates::Template;
use sqlx::postgres::PgPool;
use std::collections::HashMap;
use rocket::State;
use rocket::http::{CookieJar, Cookie};
use rocket::response::Redirect;
use serde_json::Value;
use uuid::Uuid;
use crate::tests::*;
use crate::database;
use super::{TemplateContext, style_hash};

pub static TEST_TEMPLATE: &str = r#"
{% extends "base" %}

{% block content %}
    {% set N_PAGES = data.test.pages | length %}
	<form action=
	    {% if data.page + 1 < N_PAGES %}
	        "/test/{{data.test.id}}/{{data.page+1}}"
	    {% else %}
	        "/feedback/{{data.test.id}}"
	    {% endif %} method="post">
		{% for element in data.test.pages[data.page].elements %}
		    {% if element.content.McQuestion %}
		        {% set content = element.content.McQuestion %}
                <div class="mc-question">
                    <div class="question-text">{{ content.text }}</div>
                    <div class="question-options">
                        {% for opt in content.options %}
                            <div class="question-option">
                                <input type="radio" id="{{ element.id }}_{{ loop.index-1 }}"
                                    name="questions.{{ element.id }}" value="{{ loop.index-1 }}">
                                <label for="{{ element.id }}_{{ loop.index-1 }}">{{ opt }}</label>
                            </div>
                        {% endfor %}
                    </div>
                </div>
            {% elif element.content.McQuestionVert %}
		        {% set content = element.content.McQuestionVert %}
                <div class="mc-question-vert">
                    <div class="question-text">{{ content.text }}</div>
                    <div class="question-options">
                        {% for opt in content.options %}
                            <div class="question-option">
                                <input type="radio" id="{{ element.id }}_{{ loop.index-1 }}"
                                    name="questions.{{ element.id }}" value="{{ loop.index-1 }}">
                                <label for="{{ element.id }}_{{ loop.index-1 }}">{{ opt }}</label>
                            </div>
                        {% endfor %}
                        {% if content.other %}
                            <div class="question-option">
                                <input type="radio" id="{{ element.id }}_{{ content.options | length }}"
                                    name="questions.{{ element.id }}" value="{{ content.options | length }}">
                                <label for="{{ element.id }}_{{ content.options | length }}">Other: </label>
                                <input type="text" name="questions.{{ element.id }}.other"/>
                            </div>
                        {% endif %}
                    </div>
                </div>
            {% elif element.content.Paragraph %}
                {% set content = element.content.Paragraph %}
                <div class="paragraph-question">
                    <h1> {{ content.header }} </h1>
                    <p>{{ content.paragraph }}</p>
                </div>
            {% elif element.content.CheckboxQuestion %}
		        {% set content = element.content.CheckboxQuestion %}
		        <div class="cb-question">
                    <label for="{{ element.id }}">
                        <input type="checkbox" id="{{ element.id }}" name="questions.{{ element.id }}">
                        {{ content.text }}
		            </label>
		        </div>
		    {% elif element.content.TextAreaQuestion %}
		        {% set content = element.content.TextAreaQuestion %}
		        <div class="text-area-question">
                    <h2>{{ content.header }}</h2>
                    <p>{{ content.paragraph }}</p>
                    <textarea name="questions.{{ element.id }}" rows=5 cols=50></textarea>
                </div>
            {% endif %}
		{% endfor %}
		{% if data.page + 1 < N_PAGES %}
		    <input class="submit" type="submit" value="Next Page">
		{% else %}
		    <input class="submit" type="submit" value="Get Results!">
		{% endif %}
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
    page: usize,
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

fn get_resp_map(test: &Test, page: usize, response: &Response) -> HashMap<String, Value> {
    let mut resp_map = HashMap::new();
    let page = &test.pages[page];
    for question in &page.elements {
        if let Some(value) = question.convert(&response.questions) {
            resp_map.insert(question.id.clone(), value);
        }
    }
    resp_map
}

#[post("/test/<test>/<page>", data="<response>")]
pub async fn post_test(test: &Test, page: usize, cookies: &CookieJar<'_>, response: Form<Response>, pool: &State<PgPool>) -> Redirect {
    let resp_id_cookie_name = format!("responseId[{}]", test.id);
    let response_id = cookies.get(&resp_id_cookie_name).unwrap().value().parse().unwrap();
    println!("{:?}", &response.questions);
    let resp_map = get_resp_map(test, page-1, &response);
    database::update_response(response_id, resp_map, &mut pool.acquire().await.unwrap()).await;
    Redirect::to(uri!(test(test=test, page=page)))
}

#[get("/test/<test>/<page>")]
pub async fn test(test: &Test, page: usize, cookies: &CookieJar<'_>, pool: &State<PgPool>) -> Result<Template, Redirect> {
    let resp_id_cookie_name = format!("responseId[{}]", test.id);
    let response_id = if cookies.get(&resp_id_cookie_name).is_none() {
        let gen_id = Uuid::new_v4();
        cookies.add(Cookie::new(resp_id_cookie_name.to_string(), gen_id.to_string()));
        gen_id
    }
    else {
        cookies.get(&resp_id_cookie_name).unwrap().value().parse().unwrap()
    };
    let mut conn = pool.acquire().await.unwrap();
    let resp = database::get_or_create_response(response_id, &mut conn).await;
    let show = test.pages[page].condition.eval(resp);
    if show {
        Ok(Template::render("test.html", &TemplateContext {
            title: &test.name,
            style_hash: &style_hash().await,
            data: TestContext { test: test, page: page },
        }))
    }
    else {
        if page + 1 < test.pages.len() {
            Err(Redirect::to(uri!(test(test=test, page=page+1))))
        }
        else {
            cookies.remove(Cookie::named(resp_id_cookie_name));
            Err(Redirect::to(uri!(get_feedback(test=test, id=response_id.to_string()))))
        }
    }
}

#[post("/feedback/<test>", data="<response>")]
pub async fn post_feedback(test: &Test, cookies: &CookieJar<'_>, response: Form<Response>, pool: &State<PgPool>) -> Redirect {
    let resp_id_cookie_name = format!("responseId[{}]", test.id);
    let response_id: Uuid = cookies.get(&resp_id_cookie_name).unwrap().value().parse().unwrap();
    println!("{:?}", &response.questions);
    let resp_map = get_resp_map(test, test.pages.len()-1, &response);
    database::update_response(response_id, resp_map, &mut pool.acquire().await.unwrap()).await;
    cookies.remove(Cookie::named(resp_id_cookie_name));
    Redirect::to(uri!(get_feedback(test=test, id=response_id.to_string())))
}
#[get("/feedback/<test>/<id>")]
pub async fn get_feedback(test: &Test, pool: &State<PgPool>, id: &str) -> Template {
    let response_id: Uuid = id.parse().unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let res = database::get_response(response_id, &mut conn).await;
    let mut feedback = vec![];
    for part in &test.feedback {
        feedback.push(part.score(&res));
    }
    Template::render("feedback.html", &TemplateContext {
        title: "Feedback",
        style_hash: &style_hash().await,
        data: FeedbackContext {
            feedback: &feedback
        }
    })
}