use octocrab::{markdown, models, Octocrab};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;

const FILE_TEMPLATE: &str = r#"
---
title: 'This week in Open Source at simplabs #2'
author: 'simplabs'
github: simplabs
twitter: simplabs
topic: open-source
bio: 'The simplabs team'
description:
  'A collection of work that our engineers have been carrying out in open-source
  in the past few weeks.'
og:
  image: /assets/images/posts/2022-01-11-this-week-in-os-2/og-image.png
---

Our software engineers are all active members of the open-source community and
enjoy collaborating on various projects. In this blog post, we have collected
some of the work they have done the past week!

<!--break-->

"#;

const BREAK_LINE: &str = r#"

"#;

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
}

fn process_args(pairs: Vec<Arg>) -> Args {
    let mut args = Args {
        users: vec![],
        date: String::from(""),
        date_sign: String::from(""),
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
            (name, value) => println!("Could not handle argument {} with value {}", name, value),
        }
    }

    args
}

async fn get_prs(
    octocrab: &Arc<Octocrab>,
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
        // Optional Parameters
        .send()
        .await
}

fn format_item(user_login: String, item: &Item) -> String {
    format!(
        "- [{}] [#{}]({}) {} ([@{}])",
        item.repository_name, item.issue_number, item.repository_url, item.issue_title, user_login
    )
}

#[derive(Debug)]
struct Item {
    issue_number: String,
    issue_title: String,
    repository_name: String,
    repository_url: String,
    user_login: String,
    user_url: String,
}

async fn get_user_items(octocrab: &Arc<Octocrab>, args: &Args) -> Vec<Item> {
    let mut items: Vec<Item> = vec![];

    for user in &args.users {
        let mut page = get_prs(&octocrab, user, &args.date_sign, &args.date)
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

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let octocrab = octocrab::instance();
    let args = process_args(read_args());
    let mut items = get_user_items(&octocrab, &args).await;

    items.sort_by_key(|item| item.repository_name.clone());
    let markdown_definitions = extract_definitions(&items);

    let mut file = File::create(format!("{}.md", args.date)).unwrap();
    file.write("<pre>".as_bytes());
    file.write_all(FILE_TEMPLATE.as_bytes());

    let formatted_items = items
        .into_iter()
        .map(|item| format_item(item.user_login.clone(), &item))
        .collect::<Vec<String>>()
        .join("\n");
    // let markdown = octocrab.markdown().render(&formatted_items).send().await?;
    file.write_all(formatted_items.as_bytes());
    file.write(BREAK_LINE.as_bytes());
    file.write_all(markdown_definitions.join("\n").as_bytes());
    file.write("</pre>".as_bytes());

    println!("{:?}", formatted_items);

    Ok(())
}
