extern crate getopts;

extern crate otp_cop;

use std::{env, thread};
use std::sync::{mpsc};

use otp_cop::service::{CreateServiceResult, ServiceFactory};


struct ParallelIter<T> {
    count: usize,
    pos: usize,
    rx: mpsc::Receiver<T>
}

impl<T> Iterator for ParallelIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.pos < self.count {
            return self.rx.recv().ok();
        } else {
            return None;
        }
    }
}

fn parallel<T, U, F1>(objs: Vec<T>, f1: F1) -> ParallelIter<U>
        where F1: 'static + Fn(T) -> U + Send, T: Send, U: Send {
    let (tx, rx) = mpsc::channel();
    let mut threads = vec![];
    for o in objs {
        threads.push(thread::spawn(move || {
            tx.send(f1(o));
        }));
    }

    for t in threads {
        t.join();
    }

    return ParallelIter{count: threads.len(), pos: 0, rx: rx};
}

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

    for result in parallel(services, |service| service.get_users()) {
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
        println!("");
        println!("");
    }
}
