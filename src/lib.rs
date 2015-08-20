extern crate getopts;
extern crate hyper;
extern crate rustc_serialize;


pub use github::{GithubServiceFactory};
pub use service::{CreateServiceResult, Service, ServiceFactory, GetUsersResult, GetUsersError, User};
pub use slack::{SlackServiceFactory};

pub mod github;
pub mod service;
pub mod slack;
