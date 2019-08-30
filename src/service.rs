use getopts;

pub struct GetUsersResult {
    pub service_name: String,
    pub users: Vec<User>,
}

pub struct GetUsersError {
    pub service_name: String,
    pub error_message: String,
}

pub struct User {
    pub name: String,
    pub email: Option<String>,
    pub details: Option<String>,
}

pub enum CreateServiceResult {
    None,
    MissingArguments(Vec<String>),
    Service(Box<dyn Service>),
}

pub trait ServiceFactory {
    fn add_options(&self, _: &mut getopts::Options);
    fn create_service(&self, _: &getopts::Matches) -> CreateServiceResult;
}

pub trait Service: Send + Sync {
    fn get_users(&self) -> Result<GetUsersResult, GetUsersError>;
}
