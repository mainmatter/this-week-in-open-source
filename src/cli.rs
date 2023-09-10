use chrono::{Datelike, Days, NaiveDate, NaiveWeek, Weekday};
use regex::Regex;
use serde;
use serde::Deserialize;
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::{env, time::Duration};

#[derive(PartialEq, Debug)]
pub enum CLI_CONTEXT {
    TWIOS,
    COMMENT,
}

#[derive(Debug)]
struct Arg(String, String);

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct Args {
    pub users: Vec<String>,
    pub date: String,
    pub date_sign: String,
    pub config_path: String,
    pub context: String,
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
    #[serde(default)]
    output_path: String,
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
    pub output_path: String,
    pub context: CLI_CONTEXT,
}

pub fn args() -> AppParams {
    let args = process_args(read_args());

    let cli_context = if args.context == "twios_comment" {
        CLI_CONTEXT::COMMENT
    } else {
        CLI_CONTEXT::TWIOS
    };

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
            output_path: file_config.output_path,
            context: cli_context,
        },
        Err(error) => {
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
                output_path: "".to_string(),
                context: cli_context,
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
        context: String::from(""),
    };

    for pair in pairs {
        match (pair.0.as_str(), pair.1.as_str()) {
            ("comment", _value) => {
                args.context = "twios_comment".to_string();
            }
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

    if args.date == "" {
        let now = chrono::offset::Utc::now();
        let last_week = chrono::offset::Utc::now()
            .checked_sub_days(Days::new(7))
            .unwrap()
            .naive_utc();
        args.date = format!(
            "{}..{}",
            last_week.format("%Y-%m-%d"),
            now.format("%Y-%m-%d")
        );
    }

    args
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<FileConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let config = serde_json::from_reader(reader)?;

    Ok(config)
}

// PR_COMMENT_BODY
// on:
//  issue_comment:
//    types: [created]
// - name: print title
//  env:
//    PR_COMMENT_BODY: ${{ toJSON(github.event.comment.body) }}
//  run: echo "$PR_COMMENT_BODY"

/*
Comment for the PR
Post's file path
- TWIOS_PATH /twios/ // Search for PRs after last sunday
Post's date
- TWIOS_DATE >2021-11-28 // Search for PRs after last sunday
Available categories
- TWIOS_CATEGORIES Ember,Javascript,Typescript
TWIOS_UNLABELLED
- [EmbarkStudios/spdx] UNKNOWN // unlabelled, unknown repo
- [simplabs/ember-error-route] Ember // A valid category
- [simplabs/ember-error-route] EXCLUDED // Special category to never show this again
*/

// TWIOS_CATEGORIES will be a dump of all categories in the configuration file
// once the comment entry is changed, it will need to update the JSON
// there will be no bi-directional communication
// issue_comment can update JSON but JSON can't update comment

// - Produce a PR comment that outputs the above issue_comment body
// - When issue_comment is edited, scan the changes and modify config and regenerate TWIOS file
// - Add ability for this-week to omit before/after dates and use default range of a week
// - Add ability to specify a per-post file path

struct TwiosComment {
    body: String,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Default, Debug)]
struct TwiosCommentOutput {
    labels: Vec<LabelConfig>,
    excluded: Vec<String>,
    date: String,
    file_path: String,
}

impl TwiosCommentOutput {
    fn new() -> Self {
        TwiosCommentOutput {
            labels: vec![],
            excluded: vec![],
            date: "".to_string(),
            file_path: "".to_string(),
        }
    }
}

impl TwiosComment {
    fn read(&self) -> TwiosCommentOutput {
        let mut output = TwiosCommentOutput::new();

        // (TWIOS_\w+)((\s+-\s+\[.*\]\s+\w+)+|(?:\s+(.*)))
        // (TWIOS_\w+)(((?ms)\s+-\s+\[.*\]\s+\w+)*|(?:\s+(.*)))
        let re = Regex::new(r"(TWIOS_\w+)((\s+-\s+\[.*\]\s+\w+)+|(?:\s+(.*)))").unwrap();

        for capture in re.captures_iter(&self.body) {
            let keyword = &capture[1];
            let value = &capture[2];

            match keyword {
                "TWIOS_PATH" => output.file_path = value.trim().to_string(),
                "TWIOS_DATE" => output.date = value.trim().to_string(),
                // "TWIOS_CATEGORIES" => {
                //     let categories: Vec<String> =
                //         value.split(",").map(|s| s.trim().to_string()).collect();
                // }
                "TWIOS_UNLABELLED" => {
                    let re_label = Regex::new(r"\[(?<repo>.*)\]\s+(?<label>\w+)").unwrap();
                    for line in value.split("\n") {
                        for capture in re_label.captures_iter(line) {
                            let label = &capture["label"];
                            let repo = &capture["repo"];
                            if label == "EXCLUDED" {
                                output.excluded.push(repo.to_string());
                            } else {
                                let mut found_label = false;
                                for config in &mut output.labels {
                                    if config.name == label.to_string() {
                                        config.repos.push(repo.to_string());
                                        found_label = true;
                                    }
                                }
                                if !found_label && label != "UNKNOWN" {
                                    let new_label_config = LabelConfig {
                                        name: label.to_string(),
                                        repos: vec![repo.to_string()],
                                    };
                                    output.labels.push(new_label_config);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        output
    }
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
            context: "".to_string(),
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
            context: "".to_string(),
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
            context: "".to_string(),
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
            context: "".to_string(),
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
            context: "".to_string(),
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
            context: "".to_string(),
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
            context: "".to_string(),
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
                date_sign: "".to_string(),
                output_path: "".to_string(),
                context: CLI_CONTEXT::TWIOS
            },
            args()
        );
    }

    #[test]
    fn it_reads_issue_comment() {
        let expected = TwiosComment {
            body: r#"
Post's file path
- TWIOS_PATH /twios/ 
Post's date
- TWIOS_DATE >2021-11-28 
Available categories
- TWIOS_CATEGORIES Ember,Javascript,Typescript
- TWIOS_UNLABELLED 
 - [EmbarkStudios/spdx] UNKNOWN 
 - [mainmatter/ember-simple-auth] Ember  
 - [simplabs/ember-error-route] EXCLUDED
- Doesn't catch this
            "#
            .to_string(),
        };

        assert_eq!(
            TwiosCommentOutput {
                file_path: "/twios/".to_string(),
                date: ">2021-11-28".to_string(),
                excluded: vec!["simplabs/ember-error-route".to_string()],
                labels: vec![LabelConfig {
                    name: "Ember".to_string(),
                    repos: vec!["mainmatter/ember-simple-auth".to_string()]
                }],
            },
            expected.read()
        );
    }
}
