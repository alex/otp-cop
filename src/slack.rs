use std::io::Read;

use reqwest::StatusCode;

use super::{CreateServiceResult, GetUsersError, GetUsersResult, Service, ServiceFactory, User};

#[derive(Deserialize)]
struct SlackUserListResponse {
    ok: bool,
    members: Vec<SlackUser>,
}

#[derive(Deserialize)]
struct SlackUser {
    name: String,
    deleted: bool,
    is_bot: Option<bool>,
    has_2fa: Option<bool>,
    profile: SlackProfile,
    is_owner: Option<bool>,
    is_admin: Option<bool>,
}

#[derive(Deserialize)]
struct SlackProfile {
    email: Option<String>,
}

pub struct SlackServiceFactory;

impl ServiceFactory for SlackServiceFactory {
    fn add_options(&self, opts: &mut getopts::Options) {
        opts.optopt(
            "",
            "slack-token",
            "Slack token (https://api.slack.com/web#authentication)",
            "token",
        );
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        match matches.opt_str("slack-token") {
            Some(token) => CreateServiceResult::Service(Box::new(SlackService { token })),
            None => CreateServiceResult::None,
        }
    }
}

struct SlackService {
    token: String,
}

impl Service for SlackService {
    fn get_users(&self) -> Result<GetUsersResult, GetUsersError> {
        let client = reqwest::blocking::Client::new();

        let mut response = client
            .get(&format!(
                "https://slack.com/api/users.list?token={}",
                self.token
            ))
            .send()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let mut body = String::new();
        response.read_to_string(&mut body).unwrap();

        let result = serde_json::from_str::<SlackUserListResponse>(&body).unwrap();
        assert!(result.ok);
        let users = result
            .members
            .iter()
            .filter(|user| !user.deleted && !user.is_bot.unwrap() && !user.has_2fa.unwrap())
            .map(|user| User {
                name: user.name.to_string(),
                email: Some(user.profile.email.clone().unwrap()),
                details: match (user.is_owner.unwrap(), user.is_admin.unwrap()) {
                    (true, true) => Some("Owner/Admin".to_string()),
                    (true, false) => Some("Owner".to_string()),
                    (false, true) => Some("Admin".to_string()),
                    (false, false) => None,
                },
            })
            .collect();

        Ok(GetUsersResult {
            service_name: "Slack".to_string(),
            users,
        })
    }
}
