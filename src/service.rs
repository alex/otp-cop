extern crate getopts;


pub struct GetUsersResult {
    pub service_name: String,
    pub users: Vec<User>,
}

pub struct GetUsersError {
    pub service_name: String,
    pub error_message: String,
}

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
    pub details: Option<String>,
}

pub enum CreateServiceResult {
    None,
    MissingArguments(Vec<String>),
    Service(Box<Service>),
}

pub trait ServiceFactory {
    fn add_options(&self, &mut getopts::Options);
    fn create_service(&self, &getopts::Matches) -> CreateServiceResult;
}

pub trait Service : Send + Sync {
    fn get_users(&self) -> Result<GetUsersResult, GetUsersError>;
}
