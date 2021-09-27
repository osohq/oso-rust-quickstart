use std::sync::{Arc, Mutex};

use rocket::get;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::{Build, Rocket, State};

use oso::{Oso, OsoError, PolarClass};

use crate::expenses::{Expense, DB};

#[derive(Debug)]
struct User(String);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for User {
    type Error = String;

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(user) = request.headers().get_one("user") {
            request::Outcome::Success(User(user.to_string()))
        } else {
            request::Outcome::Failure((Status::Forbidden, "Unknown User".to_owned()))
        }
    }
}

#[catch(403)]
fn not_authorized(_: &Request) -> String {
    "Not Authorized!\n".to_string()
}

#[catch(404)]
fn not_found(_: &Request) -> String {
    "Not Found!\n".to_string()
}

#[get("/expenses/<id>")]
fn get_expense(oso: &State<OsoState>, user: User, id: usize) -> Result<Option<String>, Status> {
    if let Some(expense) = DB.get(&id) {
        if oso.is_allowed(user.0, "GET", expense.clone()) {
            Ok(Some(format!("{}\n", expense)))
        } else {
            Err(Status::Forbidden)
        }
    } else {
        Ok(None)
    }
}

struct OsoState {
    oso: Arc<Mutex<Oso>>,
}

impl OsoState {
    pub fn is_allowed(&self, actor: String, action: &str, resource: Expense) -> bool {
        let guard = self.oso.lock().unwrap();
        guard
            .is_allowed(actor, action.to_string(), resource)
            .unwrap()
    }
}

pub fn oso() -> Result<Oso, OsoError> {
    let mut oso = Oso::new();

    oso.register_class(Expense::get_polar_class())?;

    oso.load_files(vec!["expenses.polar"])?;

    Ok(oso)
}

pub fn rocket(oso: Oso) -> Rocket<Build> {
    let oso_state = OsoState {
        oso: Arc::new(Mutex::new(oso)),
    };

    rocket::build()
        .mount("/", routes![get_expense])
        .manage(oso_state)
        .register("/", catchers![not_authorized, not_found])
}

pub async fn run() -> Result<(), OsoError> {
    rocket(oso()?).launch().await.unwrap();

    Ok(())
}

#[cfg(test)]
mod test {
    use super::{oso, rocket};
    use rocket::http::{Header, Status};
    use rocket::local::blocking::Client;

    #[test]
    fn get_expense_no_rules() {
        let client = Client::tracked(rocket(oso().unwrap())).expect("valid rocket instance");
        let response = client.get("/expenses/1").dispatch();
        assert_eq!(response.status(), Status::Forbidden);
    }

    #[test]
    fn get_expense_first_rule() {
        let mut oso = oso().unwrap();
        oso.load_str(
            "allow(actor: String, \"GET\", _expense: Expense) if actor.ends_with(\"@example.com\");",
        )
        .unwrap();
        let client = Client::tracked(rocket(oso)).expect("valid rocket instance");
        let mut request = client.get("/expenses/1");
        request.add_header(Header::new("user", "alice@example.com"));
        let ok_response = request.dispatch();
        assert_eq!(ok_response.status(), Status::Ok);
        let unauthorized_response = client.get("/expenses/1").dispatch();
        assert_eq!(unauthorized_response.status(), Status::Forbidden);
    }

    #[test]
    fn get_expense_second_rule() {
        let mut oso = oso().unwrap();
        oso.load_str(
            "allow(actor: String, \"GET\", expense: Expense) if expense.submitted_by = actor;",
        )
        .unwrap();
        let client = Client::tracked(rocket(oso)).expect("valid rocket instance");
        let mut request = client.get("/expenses/1");
        request.add_header(Header::new("user", "alice@example.com"));
        let ok_response = request.dispatch();
        assert_eq!(ok_response.status(), Status::Ok);

        let mut bad_request = client.get("/expenses/3");
        bad_request.add_header(Header::new("user", "alice@example.com"));
        let unauthorized_response = bad_request.dispatch();
        assert_eq!(unauthorized_response.status(), Status::Forbidden);
    }
}
