use chrono::{UTC, Duration};
use csv;
use getopts;

use hyper::client::response::{Response};

use rusoto::credentials::{AWSCredentials};
use rusoto::signature::{SignedRequest};
use rusoto::regions::{Region};

use xmltree::{Element};

use rustc_serialize::base64::{FromBase64};

use std::thread::{sleep_ms};

use super::{CreateServiceResult, Service, ServiceFactory, GetUsersResult, GetUsersError, User};

pub struct AWSServiceFactory;

impl ServiceFactory for AWSServiceFactory {
    fn add_options(&self, opts: &mut getopts::Options) {
        opts.optopt(
            "", "aws-access-key-id", "AWS Access Key Id", "access_key_id"
        );
        opts.optopt(
            "", "aws-secret-key", "AWS Secret Key", "secret_key"
        );
    }

    fn create_service(&self, matches: &getopts::Matches) -> CreateServiceResult {
        match (
            matches.opt_str("aws-access-key-id"),
            matches.opt_str("aws-secret-key")
        ) {
            (Some(access_key_id), Some(secret_key)) => CreateServiceResult::Service(Box::new(
                AWSService{access_key_id: access_key_id, secret_key: secret_key}
            )),
            (None, None) => CreateServiceResult::None,
            (access_key_id, secret_key) => {
                let mut missing = vec![];
                if access_key_id.is_none() {
                    missing.push("aws-access-key-id".to_string());
                }
                if secret_key.is_none() {
                    missing.push("aws-secret-key".to_string());
                }
                CreateServiceResult::MissingArguments(missing)
            }

        }

    }
}

struct AWSService {
    access_key_id: String,
    secret_key: String,
}

enum Action {
    GenerateCredentialReport,
    GetCredentialReport
}

impl Action {
    fn as_str(self) -> String {
        match self {
            Action::GenerateCredentialReport => "GenerateCredentialReport".to_string(),
            Action::GetCredentialReport => "GetCredentialReport".to_string()
        }
    }
}

impl Service for AWSService {
    fn get_users(&self) -> Result<GetUsersResult, GetUsersError> {
        loop {
            match self.generate_credential_report() {
                ReportResult::COMPLETE => break,
                _ => {}
            };
            sleep_ms(500); // Credential report isn't ready yet.
        }

        let aws_users = self.get_credential_report();

        return Ok(GetUsersResult{
                    service_name: "aws".to_string(),
                    users: aws_users.iter().filter_map(|user|
                        match user.mfa_active {
                            false => Some(User{
                                            name: user.user.to_string(),
                                            email: None,
                                            details: Some(user.arn.to_string())
                                          }),
                            _ => None
                        }).collect()
                    });
    }
}

#[derive(Debug)]
enum ReportResult {
    STARTED,
    COMPLETE,
}

impl ReportResult {
    fn from_str(s: &str) -> ReportResult {
        match s {
            "STARTED" => ReportResult::STARTED,
            "COMPLETE" => ReportResult::COMPLETE,
            _ => panic!("Unknown report result.")
        }
    }
}

#[derive(RustcDecodable,Debug)]
struct AWSUser {
    user: String,
    arn: String,
    user_creation_time: String,
    password_enabled: String,
    password_last_used: String,
    password_last_changed: String,
    password_next_rotation: String,
    mfa_active: bool,
    access_key_1_active: String,
    access_key_1_last_rotated: String,
    access_key_1_last_used_date: String,
    access_key_1_last_used_region: String,
    access_key_1_last_used_service: String,
    access_key_2_active: String,
    access_key_2_last_rotated: String,
    access_key_2_last_used_date: String,
    access_key_2_last_used_region: String,
    access_key_2_last_used_service: String,
    cert_1_active: String,
    cert_1_last_rotated: String,
    cert_2_active: String,
    cert_2_last_rotated: String,
}

impl AWSService {
    fn request(&self, action: Action) -> Response {
        let region = Region::UsEast1;
        let mut request = SignedRequest::new(
            "GET",
            "iam",
            &region,
            "/",
        );

        request.add_param("Version", "2010-05-08");
        request.add_param("Action", &action.as_str());

        let credentials = AWSCredentials::new(
            self.access_key_id.as_ref(),
            self.secret_key.as_ref(),
            None,
            UTC::now() + Duration::seconds(600));

        return request.sign_and_execute(&credentials)
    }

    fn generate_credential_report(&self) -> ReportResult {
        let response = self.request(Action::GenerateCredentialReport);
        let mut tree = Element::parse(response);
        let state = tree.get_mut_child("GenerateCredentialReportResult")
                        .expect("Can't find GenerateCredentialReportResult element.")
                        .get_mut_child("State")
                        .expect("Can't find State element.");

        return match state.text {
            Some(ref mut state_str) => ReportResult::from_str(state_str),
            _ => panic!("Unknown state str")
        }
    }

    fn get_credential_report(&self) -> Vec<AWSUser> {
        let response = self.request(Action::GetCredentialReport);
        let mut tree = Element::parse(response);
        let content = tree.get_mut_child("GetCredentialReportResult")
                          .expect("Can't find GetCredentialReportResult element.")
                          .get_mut_child("Content")
                          .expect("Can't get Content element.");

        let mut csv_reader = match content.text {
            Some(ref mut base64) => csv::Reader::from_bytes(base64.from_base64().unwrap()),
            _ => panic!("No content text.")
        };

        let rows = csv_reader.decode::<AWSUser>().collect::<csv::Result<Vec<_>>>().unwrap();
        return rows
    }
}
