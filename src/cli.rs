use std::env;

#[derive(Debug)]
struct Arg(String, String);

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct Args {
    pub users: Vec<String>,
    pub date: String,
    pub date_sign: String,
    pub config_path: String,
}

pub fn args() -> Args {
    process_args(read_args())
}

fn read_args() -> Vec<Arg> {
    let mut args = vec![];
    for pair in env::args().skip(1) {
        let split = pair.split("=").collect::<Vec<&str>>();

        let name = split[0].to_string();
        let value = if split.len() == 1 {
            String::from("")
        } else {
            split[1].to_string()
        };

        args.push(Arg(name, value));
    }

    args
}

fn process_args(pairs: Vec<Arg>) -> Args {
    let mut args = Args {
        users: vec![],
        date: String::from(""),
        date_sign: String::from(""),
        config_path: String::from(""),
    };

    for pair in pairs {
        match (pair.0.as_str(), pair.1.as_str()) {
            ("--users", value) => {
                args.users.append(
                    &mut value
                        .split(",")
                        .map(|user| user.to_string())
                        .collect::<Vec<String>>(),
                );
            }
            ("--date", value) => {
                args.date = value.to_string();
            }
            ("-before", _) => args.date_sign = String::from("<"),
            ("-after", _) => args.date_sign = String::from(">"),
            ("--config-path", value) => args.config_path = value.to_string(),
            (name, value) => println!("Could not handle argument {} with value {}", name, value),
        }
    }

    if args.config_path.len() == 0 {
        println!("--config-path is not provided.");
        println!("This will result with unlabelled items.");
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{process_args, Arg};

    #[test]
    fn it_processes_args() {
        let expected = Args {
            users: vec![],
            date: "".to_string(),
            date_sign: "".to_string(),
            config_path: "".to_string(),
        };

        assert_eq!(expected, process_args(vec![]));
    }

    #[test]
    fn it_processes_users_args() {
        let expected = Args {
            users: vec!["BobrImperator".to_string()],
            date: "".to_string(),
            date_sign: "".to_string(),
            config_path: "".to_string(),
        };

        assert_eq!(
            expected,
            process_args(vec![Arg(
                "--users".to_string(),
                "BobrImperator".to_string()
            )])
        );
    }

    #[test]
    fn it_processes_multiple_users_args() {
        let expected = Args {
            users: vec!["BobrImperator".to_string(), "mansona".to_string()],
            date: "".to_string(),
            date_sign: "".to_string(),
            config_path: "".to_string(),
        };

        assert_eq!(
            expected,
            process_args(vec![Arg(
                "--users".to_string(),
                "BobrImperator,mansona".to_string()
            )])
        );
    }

    #[test]
    fn it_processes_date_args() {
        let expected = Args {
            users: vec![],
            date: "2022-02-18".to_string(),
            date_sign: "".to_string(),
            config_path: "".to_string(),
        };

        assert_eq!(
            expected,
            process_args(vec![Arg("--date".to_string(), "2022-02-18".to_string())])
        );
    }

    #[test]
    fn it_processes_after_args() {
        let expected = Args {
            users: vec![],
            date: "".to_string(),
            date_sign: ">".to_string(),
            config_path: "".to_string(),
        };

        assert_eq!(
            expected,
            process_args(vec![Arg("-after".to_string(), "".to_string())])
        );
    }

    #[test]
    fn it_processes_before_args() {
        let expected = Args {
            users: vec![],
            date: "".to_string(),
            date_sign: "<".to_string(),
            config_path: "".to_string(),
        };

        assert_eq!(
            expected,
            process_args(vec![Arg("-before".to_string(), "".to_string())])
        );
    }

    #[test]
    fn it_processes_config_path_args() {
        let expected = Args {
            users: vec![],
            date: "".to_string(),
            date_sign: "".to_string(),
            config_path: "../config/location.json".to_string(),
        };

        assert_eq!(
            expected,
            process_args(vec![Arg(
                "--config-path".to_string(),
                "../config/location.json".to_string()
            )])
        );
    }
}
