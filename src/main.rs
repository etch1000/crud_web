#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

// Importing functions and structs
use diesel::{prelude::*, table, Insertable, PgConnection, Queryable};
use rocket::{fairing::AdHoc, response::Debug, serde::json::Json, State};
use rocket_sync_db_pools::database;
use serde::{Deserialize, Serialize};

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

// DTOs
table! {
    blog_posts (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

#[database("my_db")]
pub struct Db(PgConnection);

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[table_name = "blog_posts"]
struct BlogPost {
    id: i32,
    title: String,
    body: String,
    published: bool,
}

#[derive(Deserialize)]
struct Config {
    name: String,
    age: u8,
}

// Endpoints
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/random")]
fn get_random_blog_post() -> Json<BlogPost> {
    Json(BlogPost {
        id: 1,
        title: "My first post".to_string(),
        body: "This is my first post".to_string(),
        published: true,
    })
}

#[get("/<id>")]
async fn get_blog_post(connection: Db, id: i32) -> Json<BlogPost> {
    connection
        .run(move |c| blog_posts::table.filter(blog_posts::id.eq(id)).first(c))
        .await
        .map(Json)
        .expect(format!("failed to fetch the blog with id: {}", id).as_str())
}

#[get("/all")]
async fn get_all_blog_posts(connection: Db) -> Json<Vec<BlogPost>> {
    connection
        .run(|c| blog_posts::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch blog posts")
}

#[post("/new-blog", data = "<blog_post>")]
async fn create_blog_post(connection: Db, blog_post: Json<BlogPost>) -> Json<BlogPost> {
    connection
        .run(move |c| {
            diesel::insert_into(blog_posts::table)
                .values(&blog_post.into_inner())
                .get_result(c)
        })
        .await
        .map(Json)
        .expect("booo")
}

#[delete("/<id>")]
async fn delete_blog_post(connection: Db, id: i32) -> Result<Option<()>> {
    let res = connection
        .run(move |c| {
            diesel::delete(blog_posts::table)
                .filter(blog_posts::id.eq(id))
                .execute(c)
        })
        .await?;

    Ok((res == 1).then(|| ()))
}

#[get("/config")]
fn get_config(config: &State<Config>) -> String {
    format!("Hello, {}! You are {} years old!", config.name, config.age)
}

#[put("/blog-post/<id>", data = "<blog_post>")]
async fn update_blog_post(connection: Db, id: i32, blog_post: Json<BlogPost>) -> Result<()> {
    connection
        .run(move |c| {
            diesel::update(blog_posts::table.filter(blog_posts::id.eq(id)))
                .set((
                    blog_posts::title.eq(&blog_post.title),
                    blog_posts::body.eq(&blog_post.body),
                ))
                .execute(c)
        }).await?;
    Ok(())
}

// Rocket Launch
#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();

    rocket
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, get_config])
        .mount(
            "/blog-posts",
            routes![get_random_blog_post, get_blog_post, get_all_blog_posts,],
        )
        .mount("/create", routes![create_blog_post])
        .mount("/delete", routes![delete_blog_post])
        .mount("/update", routes![update_blog_post])
}
