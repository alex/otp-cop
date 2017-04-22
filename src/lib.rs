#[macro_use]
extern crate serde_derive;

extern crate getopts;
extern crate hyper;
extern crate hyper_native_tls;
extern crate serde;
extern crate serde_json;


pub use github::{GithubServiceFactory};
pub use service::{CreateServiceResult, Service, ServiceFactory, GetUsersResult, GetUsersError, User};
pub use slack::{SlackServiceFactory};

pub mod github;
pub mod service;
pub mod slack;
