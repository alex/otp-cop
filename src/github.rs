use std::io::Read;

use reqwest::StatusCode;

use super::{CreateServiceResult, GetUsersError, GetUsersResult, Service, ServiceFactory, User};

const DEFAULT_ENDPOINT: &str = "https://api.github.com";

#[derive(Deserialize)]
struct GithubError {
    documentation_url: String,
    message: String,
}

#[derive(Deserialize)]
struct GithubUser {
    login: String,
}

pub struct GithubServiceFactory;

impl ServiceFactory for GithubServiceFactory {
    fn add_options(&self, opts: &mut getopts::Options) {
        opts.optopt(
            "",
            "github-endpoint",
            &format!("Github API endpoint URL (default: {DEFAULT_ENDPOINT})"),
            "endpoint",
        );
        opts.optopt("", "github-org", "Gitub organization", "org");
        opts.optopt("", "github-username", "Gitub username", "username");
        opts.optopt("", "github-password", "Github password", "password");
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        match (
            matches.opt_str("github-endpoint"),
            matches.opt_str("github-org"),
            matches.opt_str("github-username"),
            matches.opt_str("github-password"),
        ) {
            (endpoint, Some(org), Some(username), Some(password)) => {
                CreateServiceResult::Service(Box::new(GithubService {
                    endpoint: endpoint.unwrap_or_else(|| DEFAULT_ENDPOINT.to_string()),
                    org,
                    username,
                    password,
                }))
            }
            (None, None, None, None) => CreateServiceResult::None,
            (_, org, username, password) => {
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
}

impl Service for GithubService {
    fn get_users(&self) -> Result<GetUsersResult, GetUsersError> {
        let client = reqwest::blocking::Client::new();

        let mut response = client
            .get(format!(
                "{}/orgs/{}/members?filter=2fa_disabled",
                self.endpoint, self.org
            ))
            .basic_auth(self.username.to_string(), Some(self.password.to_string()))
            .header(reqwest::header::USER_AGENT, "otp-cop/0.1.0")
            .send()
            .unwrap();
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();

        match response.status() {
            StatusCode::OK => {
                let result = serde_json::from_str::<Vec<GithubUser>>(&body).unwrap();

                Ok(GetUsersResult {
                    service_name: "Github".to_string(),
                    users: result
                        .iter()
                        .map(|user| User {
                            name: user.login.to_string(),
                            email: None,
                            details: None,
                        })
                        .collect(),
                })
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let result = serde_json::from_str::<GithubError>(&body).unwrap();

                Err(GetUsersError {
                    service_name: "Github".to_string(),
                    error_message: format!(
                        "{}\n  See {}",
                        result.message, result.documentation_url
                    ),
                })
            }
            _ => panic!("Github: unexpected status code"),
        }
    }
}
