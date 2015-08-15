use std::io::{Read};

use getopts;

use hyper::header::{Authorization, Basic, UserAgent};
use hyper::{Client};

use rustc_serialize::{json};

use super::{CreateServiceResult, Service, ServiceFactory, ServiceResult, User};


#[derive(RustcDecodable)]
struct GithubUser {
    login: String,
}


pub struct GithubServiceFactory;

impl ServiceFactory for GithubServiceFactory {
    fn add_options(&self, opts: &mut getopts:: Options) {
        opts.optopt(
            "", "github-org", "Gitub organization", "org",
        );
        opts.optopt(
            "", "github-username", "Gitub username", "username",
        );
        opts.optopt(
            "", "github-password", "Github password", "password",
        );
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        match (
            matches.opt_str("github-org"),
            matches.opt_str("github-username"),
            matches.opt_str("github-password"),
        ) {
            (Some(org), Some(username), Some(password)) => CreateServiceResult::Service(Box::new(
                GithubService{org: org, username: username, password: password}
            )),
            (None, None, None) => CreateServiceResult::None,
            (org, username, password) => {
                let mut missing = vec![];
                if org.is_none() {
                    missing.push("github-org".to_string());
                }
                if username.is_none() {
                    missing.push("github-username".to_string());
                }
                if password.is_none() {
                    missing.push("github-password".to_string());
                }
                CreateServiceResult::MissingArguments(missing)
            }
        }
    }
}

struct GithubService {
    org: String,
    username: String,
    password: String,
}

impl Service for GithubService {
    fn get_users(&self) -> ServiceResult {
        let client = Client::new();

        let mut response = client.get(
            &format!("https://api.github.com/orgs/{}/members?filter=2fa_disabled", self.org)
        ).header(
            Authorization(Basic{
                username: self.username.to_string(),
                password: Some(self.password.to_string()),
            })
        ).header(
            UserAgent("otp-cop/0.1.0".to_string())
        ).send().unwrap();
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();

        let result = json::decode::<Vec<GithubUser>>(&body).unwrap();

        return ServiceResult{
            service_name: "Github".to_string(),
            users: result.iter().map(|user| {
                User{
                    name: user.login.to_string(),
                    email: None,
                    details: None,
                }
            }).collect(),
        }
    }
}
