use crate::cli::AppParams;
use octocrab::{models, Octocrab};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum ItemMergeStatus {
    Merged,
    NotMerged,
    Unknown,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    pub issue_number: String,
    pub issue_title: String,
    pub issue_url: String,
    pub organization_name: String,
    pub repository_name: String,
    pub full_repository_name: String,
    pub repository_url: String,
    pub user_login: String,
    pub user_url: String,
    pub state: String, // "open", "closed"
    pub merge_status: ItemMergeStatus,
}

pub struct DataLoader {
    octocrab: Octocrab,
}

impl DataLoader {
    pub fn new(octocrab: Octocrab) -> Self {
        Self { octocrab }
    }

    async fn get_prs(
        &self,
        user: &String,
        date_sign: &String,
        date: &String,
    ) -> octocrab::Result<octocrab::Page<models::issues::Issue>, octocrab::Error> {
        self.octocrab
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

    pub async fn get_user_items(&self, app_params: &AppParams) -> Vec<Item> {
        let mut items: Vec<Item> = vec![];

        for user in app_params.users.clone() {
            let mut page = self
                .get_prs(&user, &app_params.date_sign, &app_params.date)
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
                        organization_name: path_parts[0].to_string(),
                        repository_name: path_parts[1].to_string(),
                        full_repository_name: format!("{}/{}", path_parts[0], path_parts[1]),
                        repository_url: repository_url_parts.join("/"),
                        state: issue.state.clone(),
                        merge_status: ItemMergeStatus::Unknown,
                    });
                }
                page = match self.octocrab.get_page(&page.next).await.unwrap() {
                    Some(next_page) => next_page,
                    None => {
                        break;
                    }
                }
            }
        }

        items
    }

    pub async fn set_item_merge_status(&self, items: &mut Vec<Item>) -> () {
        for item in items {
            match self
                .octocrab
                .pulls(item.organization_name.clone(), item.repository_name.clone())
                .is_merged(item.issue_number.parse::<u64>().unwrap())
                .await
            {
                Ok(is_merged) => {
                    if is_merged {
                        item.merge_status = ItemMergeStatus::Merged
                    } else {
                        item.merge_status = ItemMergeStatus::NotMerged
                    }
                }
                Err(_) => item.merge_status = ItemMergeStatus::Unknown,
            }
        }
    }
}
