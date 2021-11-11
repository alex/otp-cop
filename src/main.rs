use std::io::Write;
use std::sync::{mpsc, Arc};
use std::{env, process, thread};

use otp_cop::service::{CreateServiceResult, ServiceFactory};

struct ParallelIter<T> {
    count: usize,
    pos: usize,
    rx: mpsc::Receiver<T>,
}

impl<T> Iterator for ParallelIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.pos < self.count {
            self.count += 1;
            self.rx.recv().ok()
        } else {
            None
        }
    }
}

fn parallel<T, U, F1>(objs: Vec<T>, f1: F1) -> ParallelIter<U>
where
    F1: 'static + Fn(T) -> U + Send + Sync,
    T: 'static + Send,
    U: 'static + Send,
{
    let (tx, rx) = mpsc::channel();
    let count = objs.len();
    let shared_f1 = Arc::new(f1);
    for o in objs {
        let f1 = shared_f1.clone();
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(f1(o)).unwrap();
        });
    }

    ParallelIter { count, pos: 0, rx }
}

fn main() {
    let service_factories = vec![
        Box::new(otp_cop::SlackServiceFactory) as Box<dyn ServiceFactory>,
        Box::new(otp_cop::GithubServiceFactory) as Box<dyn ServiceFactory>,
    ];

    let mut opts = getopts::Options::new();

    for factory in service_factories.iter() {
        factory.add_options(&mut opts);
    }

    let matches = match opts.parse(env::args().skip(1)) {
        Ok(matches) => matches,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let mut services = vec![];

    for factory in service_factories.iter() {
        match factory.create_service(&matches) {
            CreateServiceResult::Service(s) => services.push(s),
            CreateServiceResult::MissingArguments(args) => {
                eprintln!("Missing arguments: {:?}", args);
                process::exit(1);
            }
            CreateServiceResult::None => continue,
        }
    }

    if services.is_empty() {
        eprintln!("{}", opts.usage("otp-cop: <args>"));
        process::exit(1);
    }

    let count = services.len();
    let mut error_exit = false;
    for (i, result) in parallel(services, |service| service.get_users()).enumerate() {
        match result {
            Ok(result) => {
                let header = format!("{} ({})", result.service_name, result.users.len());
                println!("{}", header);
                println!("{}", repeat_char("=", header.len()));
                println!();
                if !result.users.is_empty() {
                    error_exit = true;
                }
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
                    println!();
                    println!();
                }
            }
            Err(e) => {
                error_exit = true;
                let mut t = term::stderr().unwrap();
                writeln!(t, "{}", e.service_name).unwrap();
                writeln!(t, "{}", repeat_char("=", e.service_name.len())).unwrap();
                writeln!(t).unwrap();
                t.fg(term::color::RED).unwrap();
                writeln!(t, "{}", e.error_message).unwrap();
                t.reset().unwrap();
            }
        }
    }

    if error_exit {
        process::exit(2);
    }
}

fn repeat_char(c: &'static str, length: usize) -> String {
    c.chars().cycle().take(length).collect::<String>()
}
