extern crate getopts;


pub struct ServiceResult {
    pub service_name: String,
    pub users: Vec<User>,
}

pub struct User {
    pub name: String,
    pub email: String,
    pub details: Option<String>,
}

pub trait ServiceFactory {
    fn add_options(&self, &mut getopts::Options);
    fn create_service(&self, &getopts::Matches) -> Box<Service>;
}

pub trait Service {
    fn get_users(&self) -> ServiceResult;
}
