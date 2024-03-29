use chrono::Days;
use regex::Regex;
use serde;
use serde::Deserialize;
use serde::Serialize;
use serde_json;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(PartialEq, Debug)]
pub enum CliContext {
    TWIOS,
    COMMENT,
    UTILITY,
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
    pub context: CliContext,
    pub comment_body: String,
    pub edit: bool,
    pub dedupe: bool,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LabelConfig {
    pub name: String,
    pub repos: Vec<String>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileConfig {
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
    #[serde(default)]
    last_date: String,
    #[serde(default)]
    query_type: PullRequestQueryType,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct AppParams {
    pub exclude_closed_not_merged: bool,
    pub query_type: PullRequestQueryType,
    pub labels: Vec<LabelConfig>,
    pub header: Vec<String>,
    pub users: Vec<String>,
    pub exclude: Vec<String>,
    pub date: String,
    pub date_sign: String,
    pub config_path: String,
    pub output_path: String,
    pub context: CliContext,
    pub comment_body: String,
    pub dedupe: bool,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub enum PullRequestQueryType {
    Created,
    Merged,
}

impl Default for PullRequestQueryType {
    fn default() -> Self {
        PullRequestQueryType::Created
    }
}

impl AppParams {
    pub fn file_name(&self) -> String {
        let mut file_name = self.date.as_ref();
        let date_range = self.date.split("..").collect::<Vec<_>>();

        match date_range.get(1) {
            Some(&date) => file_name = date,
            None => (),
        }

        format!("{}{}.md", self.output_path, file_name)
    }
}

pub fn args() -> (AppParams, Option<FileConfig>) {
    let args = process_args(read_args());

    let now = chrono::offset::Utc::now();
    let last_week = chrono::offset::Utc::now()
        .checked_sub_days(Days::new(7))
        .unwrap()
        .naive_utc();
    let default_date = format!(
        "{}..{}",
        last_week.format("%Y-%m-%d"),
        now.format("%Y-%m-%d")
    );

    match read_config_from_file(args.config_path.clone()) {
        Ok(file_config) => {
            let date = if args.date.len() > 0 {
                args.date
            } else if args.edit {
                file_config.last_date.clone()
            } else {
                default_date
            };
            (
                AppParams {
                    labels: file_config.labels.clone(),
                    header: file_config.header.clone(),
                    exclude: file_config.exclude.clone(),
                    users: file_config.users.clone(),
                    exclude_closed_not_merged: file_config.exclude_closed_not_merged,
                    date,
                    date_sign: args.date_sign,
                    config_path: args.config_path,
                    output_path: file_config.output_path.clone(),
                    context: args.context,
                    comment_body: args.comment_body,
                    query_type: file_config.query_type.clone(),
                    dedupe: args.dedupe,
                },
                Some(file_config),
            )
        }
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

            (
                AppParams {
                    labels: vec![],
                    header: vec![],
                    exclude: vec![],
                    exclude_closed_not_merged: false,
                    users: args.users,
                    date: if args.date == "" {
                        default_date
                    } else {
                        args.date
                    },
                    date_sign: args.date_sign,
                    config_path: args.config_path,
                    output_path: "".to_string(),
                    context: args.context,
                    comment_body: "".to_string(),
                    query_type: PullRequestQueryType::default(),
                    dedupe: args.dedupe,
                },
                None,
            )
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
        context: CliContext::TWIOS,
        comment_body: String::from(""),
        edit: false,
        dedupe: false,
    };

    for pair in pairs {
        match (pair.0.as_str(), pair.1.as_str()) {
            ("comment", _value) => {
                args.context = CliContext::COMMENT;
            }
            ("utility", _value) => {
                args.context = CliContext::UTILITY;
            }
            ("--comment", value) => {
                args.comment_body = value.to_string();
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
            ("-edit", _) => args.edit = true,
            ("-dedupe", _) => args.dedupe = true,
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

pub fn write_config_to_file<P: AsRef<Path>>(
    path: P,
    file_config: &FileConfig,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;

    serde_json::to_writer_pretty(&file, file_config)?;

    Ok(())
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

pub struct TwiosComment {
    pub body: String,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Default, Debug)]
pub struct TwiosCommentOutput {
    pub labels: Vec<LabelConfig>,
    pub excluded: Vec<String>,
    pub date: String,
    pub file_path: String,
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

pub fn merge_with_file_config(
    comment_output: &mut TwiosCommentOutput,
    file_config: FileConfig,
) -> FileConfig {
    let mut new_config = file_config.clone();

    for label in comment_output.labels.iter_mut() {
        let in_file = new_config
            .labels
            .iter_mut()
            .find(|flabel| flabel.name == label.name);

        match in_file {
            Some(config) => {
                for repo in &label.repos {
                    if !config.repos.contains(repo) {
                        config.repos.push(repo.to_string());
                    }
                }
            }
            None => new_config.labels.push(label.clone()),
        }
    }

    for repo in &comment_output.excluded {
        if !new_config.exclude.contains(repo) {
            new_config.exclude.push(repo.to_string());
        }
    }
    new_config.last_date = comment_output.date.clone();

    new_config
}

pub fn dedupe_file_config(file_config: &mut FileConfig) {
    for label in file_config.labels.iter_mut() {
        label.repos.dedup();
    }

    file_config.exclude.dedup();
}

impl TwiosComment {
    pub fn read(&self) -> TwiosCommentOutput {
        let mut output = TwiosCommentOutput::new();

        // (TWIOS_\w+)((\s+-\s+\[.*\]\s+\w+)+|(?:\s+(.*)))
        // (TWIOS_\w+)(((?ms)\s+-\s+\[.*\]\s+\w+)*|(?:\s+(.*)))
        let re = Regex::new(r"(TWIOS_\w+)((\s+-\s+\[.*\]\s+\w*.*)+|(?:\s+(.*)))").unwrap();

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
                    let re_label =
                        Regex::new(r"\[(?<repo>.*)\]\s+(?<label>(?:\w\s?)+)\s+(?<user>@\w+)")
                            .unwrap();
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
    use std::vec;

    use super::*;

    #[test]
    fn it_processes_users_args() {
        assert_eq!(
            vec!["BobrImperator".to_string()],
            process_args(vec![Arg(
                "--users".to_string(),
                "BobrImperator".to_string()
            )])
            .users
        );
    }

    #[test]
    fn it_processes_multiple_users_args() {
        assert_eq!(
            vec!["BobrImperator".to_string(), "mansona".to_string()],
            process_args(vec![Arg(
                "--users".to_string(),
                "BobrImperator,mansona".to_string()
            )])
            .users
        );
    }

    #[test]
    fn it_processes_date_args() {
        assert_eq!(
            "2022-02-18".to_string(),
            process_args(vec![Arg("--date".to_string(), "2022-02-18".to_string())]).date
        );
    }

    #[test]
    fn it_processes_after_args() {
        assert_eq!(
            ">".to_string(),
            process_args(vec![Arg("-after".to_string(), "".to_string())]).date_sign
        );
    }

    #[test]
    fn it_processes_before_args() {
        assert_eq!(
            "<".to_string(),
            process_args(vec![Arg("-before".to_string(), "".to_string())]).date_sign
        );
    }

    #[test]
    fn it_processes_config_path_args() {
        assert_eq!(
            "../config/location.json",
            process_args(vec![Arg(
                "--config-path".to_string(),
                "../config/location.json".to_string()
            )])
            .config_path
        );
    }

    #[test]
    fn it_returns_app_params_with_defaults() {
        let (args, file_config) = args();
        assert_eq!(CliContext::TWIOS, args.context,);
        assert_eq!(None, file_config,);
    }

    #[test]
    fn it_returns_correct_file_name_given_just_date() {
        let app_params = AppParams {
            labels: vec![],
            header: vec![],
            exclude_closed_not_merged: false,
            users: vec![],
            date: "2022-06-30".to_string(),
            config_path: "".to_string(),
            context: CliContext::TWIOS,
            comment_body: "".to_string(),
            output_path: "".to_string(),
            date_sign: "".to_string(),
            exclude: vec![],
            query_type: PullRequestQueryType::Created,
            dedupe: false,
        };
        assert_eq!("2022-06-30.md", app_params.file_name());
    }

    #[test]
    fn it_returns_correct_file_name_given_date_range() {
        let app_params = AppParams {
            labels: vec![],
            header: vec![],
            exclude_closed_not_merged: false,
            users: vec![],
            date: "2022-06-23..2022-06-30".to_string(),
            config_path: "".to_string(),
            context: CliContext::TWIOS,
            comment_body: "".to_string(),
            output_path: "".to_string(),
            date_sign: "".to_string(),
            exclude: vec![],
            query_type: PullRequestQueryType::Created,
            dedupe: false,
        };
        assert_eq!("2022-06-30.md", app_params.file_name());
    }

    #[test]
    fn it_returns_correct_file_name_given_date_range_and_output_path() {
        let app_params = AppParams {
            labels: vec![],
            header: vec![],
            exclude_closed_not_merged: false,
            users: vec![],
            date: "2022-06-23..2022-06-30".to_string(),
            config_path: "".to_string(),
            context: CliContext::TWIOS,
            comment_body: "".to_string(),
            output_path: "src/twios/".to_string(),
            date_sign: "".to_string(),
            exclude: vec![],
            query_type: PullRequestQueryType::Created,
            dedupe: false,
        };
        assert_eq!("src/twios/2022-06-30.md", app_params.file_name());
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
 - [EmbarkStudios/spdx] UNKNOWN @SomeOne
 - [mainmatter/ember-simple-auth] Ember @SomeTwo  
 - [simplabs/ember-error-route] EXCLUDED @SomeThree
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

    #[test]
    fn it_merges_with_file_config() {
        let expected = TwiosComment {
            body: r#"
Post's file path
- TWIOS_PATH /twios/ 
Post's date
- TWIOS_DATE >2021-11-28 
Available categories
- TWIOS_CATEGORIES Ember,Javascript,Typescript
- TWIOS_UNLABELLED 
 - [EmbarkStudios/spdx] UNKNOWN @SomeOne
 - [mainmatter/ember-simple-auth] Ember @SomeTwo  
 - [simplabs/ember-error-route] EXCLUDED @SomeThree
- Doesn't catch this
            "#
            .to_string(),
        };

        let file_config = FileConfig {
            exclude_closed_not_merged: false,
            header: vec![],
            output_path: "".to_string(),
            exclude: vec![],
            users: vec![],
            labels: vec![],
            last_date: "".to_string(),
            query_type: PullRequestQueryType::Created,
        };

        assert_eq!(
            FileConfig {
                exclude_closed_not_merged: false,
                header: vec![],
                output_path: "".to_string(),
                exclude: vec!["simplabs/ember-error-route".to_string()],
                users: vec![],
                labels: vec![LabelConfig {
                    name: "Ember".to_string(),
                    repos: vec!["mainmatter/ember-simple-auth".to_string()]
                }],
                last_date: ">2021-11-28".to_string(),
                query_type: PullRequestQueryType::Created,
            },
            merge_with_file_config(&mut expected.read(), file_config),
        );
    }

    #[test]
    fn it_catches_labels_with_spaces() {
        let expected = TwiosComment {
            body: r#"
Post's file path
- TWIOS_PATH /twios/ 
Post's date
- TWIOS_DATE >2021-11-28 
Available categories
- TWIOS_CATEGORIES Ember,Javascript,Typescript
- TWIOS_UNLABELLED 
 - [EmbarkStudios/spdx] UNKNOWN @SomeOne
 - [mainmatter/ember-simple-auth] Ember With Spaces @SomeTwo  
 - [simplabs/ember-error-route] EXCLUDED @SomeThree
- Doesn't catch this
            "#
            .to_string(),
        };

        let file_config = FileConfig {
            exclude_closed_not_merged: false,
            header: vec![],
            output_path: "".to_string(),
            exclude: vec![],
            users: vec![],
            labels: vec![],
            last_date: "".to_string(),
            query_type: PullRequestQueryType::Created,
        };

        assert_eq!(
            FileConfig {
                exclude_closed_not_merged: false,
                header: vec![],
                output_path: "".to_string(),
                exclude: vec!["simplabs/ember-error-route".to_string()],
                users: vec![],
                labels: vec![LabelConfig {
                    name: "Ember With Spaces".to_string(),
                    repos: vec!["mainmatter/ember-simple-auth".to_string()]
                }],
                last_date: ">2021-11-28".to_string(),
                query_type: PullRequestQueryType::Created,
            },
            merge_with_file_config(&mut expected.read(), file_config),
        );
    }

    #[test]
    fn it_doesnt_push_already_exisiting_records() {
        let expected = TwiosComment {
            body: r#"
Post's file path
- TWIOS_PATH /twios/ 
Post's date
- TWIOS_DATE >2021-11-28 
Available categories
- TWIOS_CATEGORIES Ember,Javascript,Typescript
- TWIOS_UNLABELLED 
 - [EmbarkStudios/spdx] UNKNOWN @SomeOne
 - [mainmatter/ember-simple-auth] Ember @SomeTwo  
 - [simplabs/ember-error-route] EXCLUDED @SomeThree
- Doesn't catch this
            "#
            .to_string(),
        };

        let file_config = FileConfig {
            exclude_closed_not_merged: false,
            header: vec![],
            output_path: "".to_string(),
            exclude: vec!["simplabs/ember-error-route".to_string()],
            users: vec![],
            labels: vec![LabelConfig {
                name: "Ember".to_string(),
                repos: vec!["mainmatter/ember-simple-auth".to_string()],
            }],
            last_date: "".to_string(),
            query_type: PullRequestQueryType::Created,
        };

        assert_eq!(
            FileConfig {
                exclude_closed_not_merged: false,
                header: vec![],
                output_path: "".to_string(),
                exclude: vec!["simplabs/ember-error-route".to_string()],
                users: vec![],
                labels: vec![LabelConfig {
                    name: "Ember".to_string(),
                    repos: vec!["mainmatter/ember-simple-auth".to_string()]
                }],
                last_date: ">2021-11-28".to_string(),
                query_type: PullRequestQueryType::Created,
            },
            merge_with_file_config(&mut expected.read(), file_config),
        );
    }

    #[test]
    fn it_dedupes_config() {
        let mut file_config = FileConfig {
            exclude_closed_not_merged: false,
            header: vec![],
            output_path: "".to_string(),
            exclude: vec![
                "simplabs/ember-error-route".to_string(),
                "simplabs/ember-error-route".to_string(),
            ],
            users: vec![],
            labels: vec![LabelConfig {
                name: "Ember".to_string(),
                repos: vec![
                    "mainmatter/ember-simple-auth".to_string(),
                    "mainmatter/ember-simple-auth".to_string(),
                ],
            }],
            last_date: "".to_string(),
            query_type: PullRequestQueryType::Created,
        };

        dedupe_file_config(&mut file_config);

        assert_eq!(
            FileConfig {
                exclude_closed_not_merged: false,
                header: vec![],
                output_path: "".to_string(),
                exclude: vec!["simplabs/ember-error-route".to_string()],
                users: vec![],
                labels: vec![LabelConfig {
                    name: "Ember".to_string(),
                    repos: vec!["mainmatter/ember-simple-auth".to_string()]
                }],
                last_date: "".to_string(),
                query_type: PullRequestQueryType::Created,
            },
            file_config,
        );
    }
}
