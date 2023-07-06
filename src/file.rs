use crate::{document, error, authorize};
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use std::collections::HashMap;
use std::fs;
use serde_json::json;
use tracing::info;

#[derive(Debug)]
pub struct FileNames {
    names: HashMap<String, std::path::PathBuf>,
}

impl FileNames {
    pub fn new(names: HashMap<String, std::path::PathBuf>) -> Self {
        FileNames { names }
    }

    pub fn from_path(path: std::path::PathBuf) -> Result<Self, error::LinkError> {
        let files = fs::read_dir(path)?;
        let mut names = HashMap::new();
        for file in files {
            let file_path = file?.path();
            let file_stem = file_path.file_stem();
            if let Some(name) = file_stem {
                let name = name.to_owned().into_string();
                if let Ok(value) = name {
                    names.insert(value, file_path);
                }
            }
        }
        Ok(FileNames { names })
    }

    pub fn not_in(&self, links: &document::DocumentLinks) -> Self {
        let names = self.names.keys().collect::<Vec<&String>>();
        let items = links.ref_links().keys().collect::<Vec<&String>>();
        let mut diff = HashMap::<String, std::path::PathBuf>::new();
        for name in names {
            if !items.contains(&name) {
                let pair = self.names.get(name);
                if let Some(value) = pair {
                    diff.insert(name.clone(), value.clone());
                }
            }
        }
        FileNames::new(diff)
    }

    pub async fn upload(&self, url: &str, info: &document::DocInfo, user: &authorize::AuthorizedUser, id: i32) -> Result<(), error::LinkError> {
        let file_type = ".pdf";
        let convert = false;
        let show_archives = false;
        let client = reqwest::Client::new();
        info!("Client created.");

        for (name, path) in self.names() {
            let body = json!({
                "Name": "2017-2314",
                "FileType": file_type,
                "FolderId": id,
                "FileName": "2017-2314.pdf",
                "ConvertToPdf": convert,
                "File": "p:\\fila\\2017-2314.pdf",
                "IsVisible": false,
                "ShowArchives": show_archives
            });

            let res = client
                .post(url)
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "application/json")
                .header(info.headers().api_key(), user.api_key())
                .header(info.headers().partition(), user.partition())
                .header(info.headers().user_api_key(), user.user_api_key())
                .body(body.to_string())
            //     .build();
            // info!("Response url: {:?}", res.unwrap().url());
                .send()
                .await?;
            info!("Status: {}", res.status());
            let status = match &res.status() {
                &reqwest::StatusCode::OK => Ok(res.json().await?),
                _ => {
                    info!("Response: {:?}", res.text().await?);
                    Err(error::LinkError::AuthError)
                },
            };
            info!("Status: {:?}", status);
        }

        Ok(())
    }

    pub fn names(&self) -> HashMap<String, std::path::PathBuf> {
        self.names.clone()
    }
}

impl From<&document::DocumentLinks> for FileNames {
    fn from(links: &document::DocumentLinks) -> Self {
        FileNames::new(links.ref_links().clone())
    }
}
