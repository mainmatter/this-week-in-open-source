use octocrab::{models, Octocrab};
use serde;
use serde::Deserialize;
use serde_json;
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

const BREAK_LINE: &str = r#"

"#;

#[derive(Deserialize, Clone)]
struct RepoConfig {
    name: String,
    repos: Vec<String>,
    #[serde(default)]
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct FileConfig {
    labels: Vec<RepoConfig>,
    #[serde(default)]
    header: Vec<String>,
    #[serde(default)]
    users: Vec<String>,
}

fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<FileConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let config = serde_json::from_reader(reader)?;

    Ok(config)
}

#[derive(Debug)]
struct Arg(String, String);

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

#[derive(Debug)]
struct Args {
    users: Vec<String>,
    date: String,
    date_sign: String,
    config_path: String,
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

async fn get_prs(
    octocrab: &Octocrab,
    user: &String,
    date_sign: &String,
    date: &String,
) -> octocrab::Result<octocrab::Page<models::issues::Issue>, octocrab::Error> {
    octocrab
        .search()
        .issues_and_pull_requests(&format!(
            "is:pr author:{} created:{}{}",
            user.as_str(),
            date_sign.as_str(),
            date.as_str(),
        ))
        .send()
        .await
}

fn format_item(user_login: String, item: &Item) -> String {
    format!(
        "- [{}] [#{}]({}) {} ([@{}])",
        item.repository_name, item.issue_number, item.issue_url, item.issue_title, user_login
    )
}

fn format_label(repo: &RepoConfig) -> String {
    format!("## {}", repo.name)
}

#[derive(Deserialize, Debug, Clone)]
struct Item {
    issue_number: String,
    issue_title: String,
    issue_url: String,
    repository_name: String,
    repository_url: String,
    user_login: String,
    user_url: String,
}

async fn get_user_items(octocrab: &Octocrab, users: Vec<String>, args: &Args) -> Vec<Item> {
    let mut items: Vec<Item> = vec![];

    for user in users {
        let mut page = get_prs(&octocrab, &user, &args.date_sign, &args.date)
            .await
            .unwrap();

        loop {
            for issue in &page {
                let url = issue.html_url.to_string();
                let mut repository_url_parts = url.split("/").collect::<Vec<&str>>();
                let path_parts = issue
                    .html_url
                    .path()
                    .split("/")
                    .filter(|x| x.len() > 0)
                    .collect::<Vec<&str>>();

                repository_url_parts.pop(); // id
                repository_url_parts.pop(); // /pulls

                items.push(Item {
                    user_login: issue.user.login.clone(),
                    user_url: issue.user.html_url.to_string(),
                    issue_number: issue.number.to_string(),
                    issue_title: issue.title.clone(),
                    issue_url: url.to_string(),
                    repository_name: format!("{}/{}", path_parts[0], path_parts[1]),
                    repository_url: repository_url_parts.join("/"),
                });
            }
            page = match octocrab.get_page(&page.next).await.unwrap() {
                Some(next_page) => next_page,
                None => {
                    break;
                }
            }
        }
    }

    items
}

fn extract_definitions(items: &Vec<Item>) -> Vec<String> {
    let mut unique_users = HashSet::new();
    let mut unique_repositories = HashSet::new();

    for item in items {
        unique_users.insert(format!("[@{}]: {}", item.user_login, item.user_url));
        unique_repositories.insert(format!(
            "[{}]: {}",
            item.repository_name, item.repository_url
        ));
    }

    let mut unique_users = Vec::from_iter(unique_users);
    unique_users.sort();

    let mut unique_repositories = Vec::from_iter(unique_repositories);
    unique_repositories.sort();

    let mut definitions = vec![];

    definitions.append(&mut unique_users);
    definitions.append(&mut unique_repositories);

    definitions
}

async fn initialize_octocrab() -> octocrab::Result<Octocrab> {
    let (_, token) = env::vars()
        .find(|(key, _)| key == "GITHUB_PERSONAL_TOKEN")
        .unwrap_or((String::from("DEFAULT"), String::from("")));

    if token.len() > 0 {
        Octocrab::builder().personal_token(token).build()
    } else {
        Octocrab::builder().build()
    }
}

fn match_items_with_labels<'a>(
    repos: &'a mut Vec<RepoConfig>,
    items: &Vec<Item>,
) -> (&'a Vec<RepoConfig>, Vec<Item>) {
    let mut unknown_items: Vec<Item> = vec![];

    for item in items {
        let label = repos
            .into_iter()
            .find(|label| label.repos.contains(&item.repository_name));

        match label {
            Some(label) => {
                label.items.push(item.clone());
            }
            None => unknown_items.push(item.clone()),
        }
    }

    (repos, unknown_items)
}

fn format_items(items: &Vec<Item>) -> Vec<String> {
    items
        .into_iter()
        .map(|item| format_item(item.user_login.clone(), &item))
        .collect::<Vec<String>>()
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let octocrab = initialize_octocrab().await?;

    let args = process_args(read_args());
    match read_config_from_file(args.config_path.clone()) {
        Ok(config) => {
            let mut repos = config.labels.clone();
            repos.sort_by_key(|label| label.name.clone());

            let users = if config.users.len() > 0 {
                config.users.clone()
            } else {
                args.users.clone()
            };
            let mut items = get_user_items(&octocrab, users, &args).await;

            items.sort_by_key(|item| item.repository_name.clone());
            let markdown_definitions = extract_definitions(&items);

            let mut file = File::create(format!("{}.md", args.date)).unwrap();

            let (labels, unknown_items) = match_items_with_labels(&mut repos, &items);

            let mut content: Vec<String> = vec![];

            for (i, label) in labels.iter().filter(|i| i.items.len() > 0).enumerate() {
                if i > 0 {
                    content.push(String::from(""));
                }
                content.push(format_label(&label));
                content.push(String::from(""));
                content.append(&mut format_items(&label.items));
            }

            if unknown_items.len() > 0 {
                content.push(String::from(""));
                content.push(String::from("## Unknown"));
                content.push(String::from(""));
                content.append(&mut format_items(&unknown_items));
            }

            file.write_all(config.header.join("\n").as_bytes());
            file.write_all(content.join("\n").as_bytes());
            file.write(BREAK_LINE.as_bytes());
            file.write_all(markdown_definitions.join("\n").as_bytes());
        }
        Err(e) => {
            println!(
                "Couldn't open configuration file '--config-path={}'",
                args.config_path.clone()
            );
            println!("{}", e);
            let mut items = get_user_items(&octocrab, args.users.clone(), &args).await;

            items.sort_by_key(|item| item.repository_name.clone());
            let markdown_definitions = extract_definitions(&items);

            let mut file = File::create(format!("{}.md", args.date)).unwrap();

            let mut content: Vec<String> = vec![];

            content.append(&mut format_items(&items));

            file.write_all(content.join("\n").as_bytes());
            file.write(BREAK_LINE.as_bytes());
            file.write_all(markdown_definitions.join("\n").as_bytes());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_formats_items() {
        let items: Vec<Item> = vec![
            Item {
                issue_number: "63".to_string(),
                issue_title: "Update nan".to_string(),
                issue_url: "https://github.com/atom/keyboard-layout/pull/63".to_string(),
                repository_name: "atom/keyboard-layout".to_string(),
                repository_url: "https://github.com/atom/keyboard-layout".to_string(),
                user_login: "mansona".to_string(),
                user_url: "https://github.com/mansona".to_string(),
            },
            Item {
                issue_number: "798".to_string(),
                issue_title: "Ember 4 compatibility".to_string(),
                issue_url: "https://github.com/ember-engines/ember-engines/pull/798".to_string(),
                repository_name: "ember-engines/ember-engines".to_string(),
                repository_url: "https://github.com/ember-engines/ember-engines".to_string(),
                user_login: "BobrImperator".to_string(),
                user_url: "https://github.com/BobrImperator".to_string(),
            },
        ];

        let expected = vec![
            "- [atom/keyboard-layout] [#63](https://github.com/atom/keyboard-layout/pull/63) Update nan ([@mansona])",
            "- [ember-engines/ember-engines] [#798](https://github.com/ember-engines/ember-engines/pull/798) Ember 4 compatibility ([@BobrImperator])",
        ];
        assert_eq!(expected, format_items(&items));
    }
}
