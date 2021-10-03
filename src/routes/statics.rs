use rocket::fs::NamedFile;

#[get("/static/style.css")]
pub async fn style() -> Option<NamedFile> {
    NamedFile::open("static/style.css").await.ok()
}
