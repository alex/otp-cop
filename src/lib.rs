#[macro_use]
extern crate serde_derive;

pub use crate::github::GithubServiceFactory;
pub use crate::service::{
    CreateServiceResult, GetUsersError, GetUsersResult, Service, ServiceFactory, User,
};
pub use crate::slack::SlackServiceFactory;

pub mod github;
pub mod service;
pub mod slack;
