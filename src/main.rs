#[macro_use] extern crate rocket;
use rocket::serde::Serialize;
use rocket_dyn_templates::{Template};
use rocket_dyn_templates::tera::Tera;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct TemplateContext<'r> {
	title: &'r str
}

#[get("/")]
fn index() -> Template {
	Template::render("index", &TemplateContext{
		title: "Index"
	})
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
                   .attach(Template::fairing())
}
