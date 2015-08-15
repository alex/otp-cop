use getopts;

use super::{Service, ServiceFactory, ServiceResult, User};


pub struct GithubServiceFactory;

impl ServiceFactory for GithubServiceFactory {
    fn add_options(&self, opts: &mut getopts:: Options) {
    }

    fn create_service(&self, matches: &getopts::Matches) -> Box<Service> {
        return Box::new(GithubService)
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
