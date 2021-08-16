#[macro_use]
extern crate rocket;

mod expenses;
mod server;

#[tokio::main]
async fn main() {
    if let Err(err) = server::run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
