use std::sync::{Arc, Mutex};

use rocket::http::Status;
use rocket::{get, routes};
use rocket::{Build, Rocket, State};

use oso::{Oso, OsoError, PolarClass};

use crate::models::{Repository, User};

#[get("/repo/<name>")]
fn get_repo(oso: &State<OsoState>, name: &str) -> Result<Option<String>, Status> {
    let Some(repo) = Repository::by_name(name) else {
        return Err(Status::NotFound);
    };
    if oso.is_allowed(User::get_current_user(), "read", repo.clone()) {
        return Ok(Some(format!(
            "<h1>A Repo</h1><p>Welcome to repo {}</p>",
            repo.name
        )));
    }
    Err(Status::Forbidden)
}

struct OsoState {
    oso: Arc<Mutex<Oso>>,
}

impl OsoState {
    pub fn is_allowed(&self, actor: User, action: &str, resource: Repository) -> bool {
        let guard = self.oso.lock().unwrap();
        guard
            .is_allowed(actor, action.to_string(), resource)
            .unwrap()
    }
}

pub fn oso() -> Result<Oso, OsoError> {
    let mut oso = Oso::new();

    oso.register_class(User::get_polar_class())?;
    oso.register_class(
        Repository::get_polar_class_builder()
            .with_equality_check()
            .build(),
    )?;

    oso.load_files(vec!["main.polar"])?;

    Ok(oso)
}

pub fn rocket(oso: Oso) -> Rocket<Build> {
    let oso_state = OsoState {
        oso: Arc::new(Mutex::new(oso)),
    };

    rocket::build()
        .mount("/", routes![get_repo])
        .manage(oso_state)
}

#[cfg(test)]
mod test {
    use super::{oso, rocket};
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    fn http_client() -> Client {
        Client::tracked(rocket(oso().unwrap())).expect("valid rocket instance")
    }

    #[test]
    fn test_basic() {
        let client = http_client();
        let resp = client.get("/repo/oso").dispatch();
        assert_eq!(resp.status(), Status::Forbidden);

        let resp = client.get("/repo/gmail").dispatch();
        assert_eq!(resp.status(), Status::Ok);
    }
}
