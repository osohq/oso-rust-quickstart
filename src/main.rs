#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod expenses;
mod server;

fn main() {
    server::run();
}
