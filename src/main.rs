extern crate getopts;

extern crate otp_cop;

use std::{env, thread};
use std::sync::{mpsc};

use otp_cop::service::{CreateServiceResult, ServiceFactory};


fn main() {
    let service_factories = vec![
        Box::new(otp_cop::SlackServiceFactory) as Box<ServiceFactory>,
        Box::new(otp_cop::GithubServiceFactory) as Box<ServiceFactory>,
    ];

    let mut opts = getopts::Options::new();

    for factory in service_factories.iter() {
        factory.add_options(&mut opts);
    }

    let matches = match opts.parse(env::args().skip(1)) {
        Ok(matches) => matches,
        Err(e) => panic!(e.to_string()),
    };

    let mut services = vec![];

    for factory in service_factories.iter() {
        match factory.create_service(&matches) {
            CreateServiceResult::Service(s) => services.push(s),
            CreateServiceResult::MissingArguments(arg) => panic!(format!("Missing arguments: {:?}", arg)),
            CreateServiceResult::None => continue,
        }
    }

    if services.is_empty() {
        print!("{}", opts.usage("otp-cop: <args>"));
    }

    let (tx, rx) = mpsc::channel();
    let count = services.len();
    for service in services {
        let tx = tx.clone();
        thread::spawn(move || {
            let result = service.get_users();
            tx.send(result).unwrap();
        });
    }

    for (i, result) in rx.iter().enumerate() {
        println!("{}", result.service_name);
        println!("{}", "=".chars().cycle().take(result.service_name.len()).collect::<String>());
        println!("");
        for user in result.users {
            let email = match user.email {
                Some(email) => format!(" ({})", email),
                None => "".to_string(),
            };
            let details = match user.details {
                Some(details) => format!(" -- {}", details),
                None => "".to_string(),
            };
            println!("@{}{}{}", user.name, email, details);
        }
        if i + 1 != count {
            println!("");
            println!("");
        } else {
            break;
        }
    }
}
