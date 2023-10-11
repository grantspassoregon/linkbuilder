use crate::{authorize, document, error};
use data_encoding::BASE64;
use pdf::file;
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
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

    pub async fn upload(
        &self,
        url: &str,
        info: &document::DocInfo,
        user: &authorize::AuthorizedUser,
        id: i32,
    ) -> Result<(), error::LinkError> {
        let client = reqwest::Client::new();
        info!("Client created.");

        for (name, path) in self.names() {
            let mut file = File::open(path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            // let file = file::FileOptions::cached().open(&path).unwrap();
            // if let Some(ref info) = file.trailer.info_dict {
            //     let title = info.get("Title").and_then(|p| p.to_string_lossy().ok());
            //     let author = info.get("Author").and_then(|p| p.to_string_lossy().ok());
            // }
            let enc = BASE64.encode(&data);
            // let mut backup = File::create("p:/encode.txt")?;
            // backup.write_all(&enc.as_bytes())?;

            let body = json!({
                "Name": name,
                "FileName": format!("{}.pdf", name),
                "File": format!("{}", enc),
                "FolderId": id,
                "Status": "Published",
                "ConvertToPdf": "false",
                "IsVisible": "false",
            });
            info!("Body is: {}", &body);

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
                &reqwest::StatusCode::CREATED => Ok(res.json().await?),
                _ => {
                    info!("Response: {:?}", res.text().await?);
                    Err(error::LinkError::AuthError)
                }
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
