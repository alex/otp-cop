use std::io::{Read};
use std::fmt;
use std;
use regex::Regex;

use getopts;

use hyper;
use hyper::{Client};
use hyper::header::{Authorization, Basic, UserAgent, Header, HeaderFormat};
use hyper::status::{StatusCode};

use rustc_serialize::{json};

use super::{CreateServiceResult, Service, ServiceFactory, GetUsersResult, GetUsersError, User};

const DEFAULT_ENDPOINT: &'static str = "https://api.github.com";

#[derive(RustcDecodable)]
struct GithubError {
    documentation_url: String,
    message: String,
}

#[derive(RustcDecodable)]
struct GithubUser {
    login: String,
}

#[derive(RustcDecodable)]
struct GithubUserInfo{
    name: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Clone)]
struct GithubLinkHeader {
    next: Option<String>,
}
impl Header for GithubLinkHeader {
    fn header_name() -> &'static str {
        "Link"
    }

    fn parse_header(raw: &[Vec<u8>]) -> hyper::Result<GithubLinkHeader>
    {
        let line = raw.iter().next().map(|s| std::str::from_utf8 (s).unwrap());
        let re = Regex::new(r##"<(?P<url>.+?)>; rel="next""##).unwrap();
        let caps = re.captures(line.unwrap());
        let next = match caps {
            Some(cap) => Some(cap.name("url").unwrap().to_string()),
            None => None,
        };
        return Ok(GithubLinkHeader{
            next: next,
        });
    }
}
impl HeaderFormat for GithubLinkHeader {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GithubLinkHeader")
            .field("Next", &self.next)
            .finish()
    }
}

pub struct GithubServiceFactory;

impl ServiceFactory for GithubServiceFactory {
    fn add_options(&self, opts: &mut getopts::Options) {
        opts.optopt(
            "",
            "github-endpoint",
            &format!("Github API endpoint URL (default: {})", DEFAULT_ENDPOINT),
            "endpoint"
        );
        opts.optopt(
            "", "github-org", "Gitub organization", "org",
        );
        opts.optopt(
            "", "github-username", "Gitub username", "username",
        );
        opts.optopt(
            "", "github-password", "Github password", "password",
        );
        opts.optflag(
            "", "github-namecheck", "Check name compliance not 2fa",
        );
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        match (
            matches.opt_str("github-endpoint"),
            matches.opt_str("github-org"),
            matches.opt_str("github-username"),
            matches.opt_str("github-password"),
            matches.opt_present("github-namecheck"),
        ) {
            (endpoint, Some(org), Some(username), Some(password), namecheck) => CreateServiceResult::Service(Box::new(
                GithubService{
                    endpoint: endpoint.unwrap_or(DEFAULT_ENDPOINT.to_string()),
                    org: org,
                    username: username,
                    password: password,
                    namecheck: namecheck,
                }
            )),
            (None, None, None, None, _) => CreateServiceResult::None,
            (_, org, username, password, _) => {
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
    endpoint: String,
    org: String,
    username: String,
    password: String,
    namecheck: bool,
}

impl GithubService {
    fn get_user_info(&self, login: String) -> GithubUserInfo {
        let client = Client::new();

        let mut response = client.get(
            &format!("{}/users/{}", self.endpoint, login)
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

        match response.status {
            StatusCode::Ok => return json::decode::<GithubUserInfo>(&body).unwrap(),
            _ => panic!("Github: No user found"),
        }

    }

    fn get_users_helper(&self, link: &GithubLinkHeader, prev_results: GetUsersResult ) -> Result<GetUsersResult, GetUsersError> {
        match link.next {
            None => return Ok(prev_results),
            Some(ref url) => {
                let client = Client::new();

                let mut response = client.get(&format!("{}", url)
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

                match response.status {
                    StatusCode::Ok => {
                        let result = json::decode::<Vec<GithubUser>>(&body).unwrap();
                        let mut all_users = prev_results.users.clone();

                        let mut new_users: Vec<User> = result.iter().map(|user| {
                            let info = self.get_user_info(user.login.to_string());
                            User{
                                name: user.login.to_string(),
                                email: info.email,
                                details: info.name,
                            }
                        }).filter(
                            |u| u.details.is_none() || !self.namecheck
                        ).collect::<Vec<User>>();

                        all_users.append(&mut new_users);
                        let results = GetUsersResult{
                            service_name: "Github".to_string(),
                            users: all_users,
                        };

                        match response.headers.get::<GithubLinkHeader>() {
                            Some(links) => return self.get_users_helper(links, results),
                            None => return Ok(results),
                        }
                    },
                    StatusCode::UnprocessableEntity => {
                        let result = json::decode::<GithubError>(&body).unwrap();

                        return Err(GetUsersError{
                            service_name: "Github".to_string(),
                            error_message: format!("{}\n  See {}", result.message, result.documentation_url),
                        });
                    },
                    _ => panic!("Github: unexpected status code"),
                }
            }
        }
    }
}

impl Service for GithubService {
    fn get_users(&self) -> Result<GetUsersResult, GetUsersError> {
        let url = match self.namecheck {
            true => format!("{}/orgs/{}/members?filter=all", self.endpoint, self.org),
            false => format!("{}/orgs/{}/members?filter=2fa_disabled", self.endpoint, self.org),
        };
        return self.get_users_helper(
            &GithubLinkHeader{ next: Some(url) },
            GetUsersResult{
                service_name: "Github".to_string(),
                users: Vec::new(),
            }
        );
    }
}
