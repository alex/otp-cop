#[macro_use]
extern crate serde_derive;

extern crate getopts;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

pub use crate::github::GithubServiceFactory;
pub use crate::service::{
    CreateServiceResult, GetUsersError, GetUsersResult, Service, ServiceFactory, User,
};
pub use crate::slack::SlackServiceFactory;

pub mod github;
pub mod service;
pub mod slack;
