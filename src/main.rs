#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rand;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use rocket::request::Form;
use rocket::State;
use rocket_contrib::templates::Template;
use std::sync::Mutex;

use rocket::response::Redirect;

use std::collections::HashMap;

#[derive(Serialize)]
struct TemplateContext {
    id: String,
}

#[derive(FromForm)]
struct RedirectInput {
    url: String,
}

struct StoredRedirects {
    list: Mutex<HashMap<String, String>>,
}

#[get("/?<id>")]
fn index(id : Option<String>) -> Template {
    let context = TemplateContext {
        id:id.unwrap_or("".to_string()),
    };
    return Template::render("index", &context);
    
}

#[post("/", data = "<data>")]
fn add(data: Form<RedirectInput>, state: State<StoredRedirects>) -> Redirect {
    let random_id: String = thread_rng().sample_iter(&Alphanumeric).take(5).collect();
    println!("Got something: {} => {}", random_id, data.url);
    let mut lock = state.list.lock().unwrap();
    lock.insert(random_id.to_string(), data.url.to_string());
    Redirect::to(uri!(index : id = random_id.to_string() ))
}

#[get("/<id>")]
fn redirect(id: String, state: State<StoredRedirects>) -> Redirect {
    let mut lock = state.list.lock().unwrap();
    if !lock.contains_key(&id) {
        println!("Key {} not found", id);
        return Redirect::to(uri!(index: _));
    }
    let url = lock.remove(&id).unwrap().to_string();
    println!("Key {} found, redirecting to {}", id, url);
    return Redirect::to(url);
}

fn main() {
    let state_bucket = StoredRedirects {
        list: Mutex::new(HashMap::new()),
    };

    rocket::ignite()
        .mount("/", routes![index, add, redirect])
        .manage(state_bucket)
        .attach(Template::fairing())
        .launch();
}
