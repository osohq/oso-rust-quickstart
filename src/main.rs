mod models;
mod server;

#[rocket::launch]
fn launch() -> _ {
    server::rocket(server::oso().expect("Valid oso instance"))
}
