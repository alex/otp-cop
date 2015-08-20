use std::io::{Read};

use getopts;

use hyper::{Client};
use hyper::status::{StatusCode};

use rustc_serialize::{json};

use super::{CreateServiceResult, Service, ServiceFactory, ServiceResult, User};


#[derive(RustcDecodable)]
struct SlackUserListResponse {
    ok: bool,
    members: Vec<SlackUser>,
}

#[derive(RustcDecodable)]
struct SlackUser {
    name: String,
    deleted: bool,
    is_bot: Option<bool>,
    has_2fa: Option<bool>,
    profile: SlackProfile,
    is_owner: Option<bool>,
    is_admin: Option<bool>,
}

#[derive(RustcDecodable)]
struct SlackProfile {
    email: Option<String>,
}

pub struct SlackServiceFactory;

impl ServiceFactory for SlackServiceFactory {
    fn add_options(&self, opts: &mut getopts::Options) {
        opts.optopt(
            "", "slack-token", "Slack token (https://api.slack.com/web#authentication)", "token"
        );
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        match matches.opt_str("slack-token") {
            Some(token) => CreateServiceResult::Service(Box::new(SlackService{
                token: token
            })),
            None => CreateServiceResult::None,
        }
    }
}

struct SlackService {
    token: String,
}

impl Service for SlackService {
    fn get_users(&self) -> ServiceResult {
        let client = Client::new();

        let mut response = client.get(
            &format!("https://slack.com/api/users.list?token={}", self.token)
        ).send().unwrap();
        assert_eq!(response.status, StatusCode::Ok);
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();

        let result = json::decode::<SlackUserListResponse>(&body).unwrap();
        assert!(result.ok);
        let users = result.members.iter().filter(|user| {
            match user.deleted {
                true => false,
                false => match user.is_bot.unwrap() {
                    true => false,
                    false => !user.has_2fa.unwrap(),
                }
            }
        }).map(|user|
            User{
                name: user.name.to_string(),
                email: Some(user.profile.email.clone().unwrap()),
                details: match (user.is_owner.unwrap(), user.is_admin.unwrap()) {
                    (true, true) => Some("Owner/Admin".to_string()),
                    (true, false) => Some("Owner".to_string()),
                    (false, true) => Some("Admin".to_string()),
                    (false, false) => None
                }
            }
        ).collect();

        return ServiceResult{
            service_name: "Slack".to_string(),
            users: users,
        }
    }
}
