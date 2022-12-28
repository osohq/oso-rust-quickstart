use std::collections::HashMap;

use lazy_static::lazy_static;
use oso::PolarClass;

lazy_static! {
    static ref REPOS_DB: HashMap<&'static str, Repository> = {
        let mut db = HashMap::with_capacity(3);
        db.insert("gmail", Repository::new("gmail", false));
        db.insert("react", Repository::new("react", true));
        db.insert("oso", Repository::new("oso", false));
        db
    };
    static ref USERS_DB: HashMap<&'static str, User> = {
        let mut db = HashMap::with_capacity(3);
        db.insert(
            "larry",
            User::new(vec![Role::new(
                "admin",
                Repository::by_name("gmail").unwrap(),
            )]),
        );

        db.insert(
            "anne",
            User::new(vec![Role::new(
                "maintainer",
                Repository::by_name("react").unwrap(),
            )]),
        );
        db.insert(
            "graham",
            User::new(vec![Role::new(
                "contributor",
                Repository::by_name("oso").unwrap(),
            )]),
        );
        db
    };
}

#[derive(Clone, Debug, PartialEq, Eq, PolarClass)]
pub struct Repository {
    #[polar(attribute)]
    pub name: String,
    pub is_public: bool,
}

impl Repository {
    pub fn new(name: &str, is_public: bool) -> Self {
        Self {
            name: name.to_string(),
            is_public,
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        REPOS_DB.get(name).map(Clone::clone)
    }
}

#[derive(Clone, Debug, PolarClass)]
pub struct Role {
    #[polar(attribute)]
    pub name: String,
    #[polar(attribute)]
    pub repository: Repository,
}

impl Role {
    pub fn new(name: &str, repository: Repository) -> Self {
        Self {
            name: name.to_owned(),
            repository,
        }
    }
}

#[derive(Clone, Debug, PolarClass)]
pub struct User {
    #[polar(attribute)]
    pub roles: Vec<Role>,
}

impl User {
    pub const fn new(roles: Vec<Role>) -> Self {
        Self { roles }
    }

    pub fn get_current_user() -> Self {
        USERS_DB.get("larry").unwrap().clone()
    }
}
