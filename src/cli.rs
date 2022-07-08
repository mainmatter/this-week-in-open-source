use serde;
use serde::Deserialize;
use serde_json;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

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

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Clone, Debug)]
pub struct LabelConfig {
    pub name: String,
    pub repos: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct FileConfig {
    labels: Vec<LabelConfig>,
    #[serde(default)]
    header: Vec<String>,
    #[serde(default)]
    users: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    exclude_closed_not_merged: bool,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct AppParams {
    pub exclude_closed_not_merged: bool,
    pub labels: Vec<LabelConfig>,
    pub header: Vec<String>,
    pub users: Vec<String>,
    pub exclude: Vec<String>,
    pub date: String,
    pub date_sign: String,
    pub config_path: String,
}

pub fn args() -> AppParams {
    let args = process_args(read_args());

    match read_config_from_file(args.config_path.clone()) {
        Ok(file_config) => AppParams {
            labels: file_config.labels,
            header: file_config.header,
            exclude: file_config.exclude,
            users: file_config.users,
            exclude_closed_not_merged: file_config.exclude_closed_not_merged,
            date: args.date,
            date_sign: args.date_sign,
            config_path: args.config_path,
        },
        Err(error) => {
            println!("");
            if args.config_path.len() == 0 {
                println!("--config-path is not provided.");
                println!("This will result with unlabelled items.");
            } else {
                println!("There was a problem reading your config file.");
                println!("Check if your config file is correct and valid.");
                println!("");
                println!("{:?}", error);
            }

            AppParams {
                labels: vec![],
                header: vec![],
                exclude: vec![],
                exclude_closed_not_merged: false,
                users: args.users,
                date: args.date,
                date_sign: args.date_sign,
                config_path: args.config_path,
            }
        }
    }
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

    args
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<FileConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let config = serde_json::from_reader(reader)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn it_returns_app_params_with_defaults() {
        assert_eq!(
            AppParams {
                exclude_closed_not_merged: false,
                labels: vec![],
                header: vec![],
                users: vec![],
                exclude: vec![],
                config_path: "".to_string(),
                date: "".to_string(),
                date_sign: "".to_string()
            },
            args()
        );
    }
}
