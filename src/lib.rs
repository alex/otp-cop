extern crate getopts;
extern crate hyper;
extern crate rustc_serialize;


pub use github::{GithubServiceFactory};
pub use service::{Service, ServiceFactory, ServiceResult, User};
pub use slack::{SlackServiceFactory};

pub mod github;
pub mod service;
pub mod slack;
