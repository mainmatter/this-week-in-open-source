use octocrab::{markdown, models, Octocrab};
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
}

#[derive(Debug)]
struct UserItem {
    login: String,
    items: Vec<Item>,
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let octocrab = octocrab::instance();
    println!("{:?}", read_args());
    println!("{:?}", process_args(read_args()));
    let args = process_args(read_args());
    // Returns the first page of all issues.
    // Go through every page of issues. Warning: There's no rate limiting so
    // be careful.
    //
    let mut user_items: Vec<UserItem> = vec![];

    for user in &args.users {
        let mut page = get_prs(&octocrab, user, &args.date_sign, &args.date).await?;
        let mut user_item = UserItem {
            login: user.to_string(),
            items: vec![],
        };

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

                user_item.items.push(Item {
                    user_login: issue.user.login.clone(),
                    issue_number: issue.number.to_string(),
                    issue_title: issue.title.clone(),
                    repository_name: format!("{}/{}", path_parts[0], path_parts[1]),
                    repository_url: repository_url_parts.join("/"),
                });
            }
            page = match octocrab.get_page(&page.next).await? {
                Some(next_page) => next_page,
                None => {
                    user_items.push(user_item);
                    break;
                }
            }
        }
    }

    let mut file = File::create(format!("{}.html", args.date)).unwrap();
    file.write_all(FILE_TEMPLATE.as_bytes());
    let content = String::from("");

    for user_item in &user_items {
        let items = &user_item.items;
        let formatted_items = items
            .into_iter()
            .map(|item| format_item(user_item.login.clone(), &item))
            .collect::<Vec<String>>()
            .join("\n");

        let markdown = octocrab.markdown().render(&formatted_items).send().await?;
        file.write_all(markdown.as_bytes());
        file.write("<pre>".as_bytes());
        file.write_all(formatted_items.as_bytes());
        file.write("</pre>".as_bytes());
    }
    println!("{:?}", user_items);

    // let octocrab = Octocrab::default();
    // let mut page = octocrab
    //     .pulls("simplabs", "qunit-dom")
    //     .list()
    //     .per_page(5)
    //     .send()
    //     .await?;

    // let number_of_pages = page.number_of_pages();

    // for page in number_of_pages.drain() {

    // }

    // match number_of_pages {
    //     Some(a) => println!(":)(, {}", a),
    //     None => println!("Nothing"),
    // }

    // let mut prs = page.take_items();
    // println!("REEE");
    // for pr in prs.drain(..) {
    //     println!("{}, yay", pr.url);
    // }

    // let mut current_page = octocrab
    //     .orgs("rust-lang")
    //     .list_repos()
    //     .repo_type(params::repos::Type::Sources)
    //     .per_page(100)
    //     .send()
    //     .await?;
    // let mut prs = current_page.take_items();

    // while let Ok(Some(mut new_page)) = octocrab.get_page(&current_page.next).await {
    //     prs.extend(new_page.take_items());

    //     for pr in prs.drain(..) {
    //         println!("{:?}", pr);
    //     }

    //     current_page = new_page;
    // }

    Ok(())
}
