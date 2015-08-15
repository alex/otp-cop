use getopts;

use super::{CreateServiceResult, Service, ServiceFactory, ServiceResult, User};


pub struct GithubServiceFactory;

impl ServiceFactory for GithubServiceFactory {
    fn add_options(&self, opts: &mut getopts:: Options) {
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        return CreateServiceResult::None;
    }
}

struct GithubService;

impl Service for GithubService {
    fn get_users(&self) -> ServiceResult {
        return ServiceResult{
            service_name: "Github".to_string(),
            users: vec![],
        }
    }
}
