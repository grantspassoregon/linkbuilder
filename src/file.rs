use crate::{authorize, document, error};
use data_encoding::BASE64;
use indicatif::ProgressBar;
use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use tracing::{trace, warn};

#[derive(Debug)]
pub struct FileNames {
    names: HashMap<String, std::path::PathBuf>,
}

impl FileNames {
    pub fn new(names: HashMap<String, std::path::PathBuf>) -> Self {
        FileNames { names }
    }

    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, error::LinkError> {
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
    ) -> Result<Vec<String>, error::LinkError> {
        let mut rec = Vec::new();
        let client = reqwest::Client::new();
        trace!("Upload client created.");

        let style = indicatif::ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {'Uploading files.'}",
        )
        .unwrap();
        let bar = ProgressBar::new(self.names().len() as u64);
        bar.set_style(style);
        for (name, path) in self.names() {
            let mut file = File::open(path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            let enc = BASE64.encode(&data);

            let body = json!({
                "Name": name,
                "FileName": format!("{}.pdf", name),
                "File": format!("{}", enc),
                "FolderId": id,
                "Status": "Published",
                "ConvertToPdf": "false",
                "IsVisible": "false",
            });

            let res = client
                .post(url)
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "application/json")
                .header(info.headers().api_key(), user.api_key())
                .header(info.headers().partition(), user.partition())
                .header(info.headers().user_api_key(), user.user_api_key())
                .body(body.to_string())
                .send()
                .await?;
            bar.inc(1);
            match &res.status() {
                &reqwest::StatusCode::OK => {
                    rec.push(res.json().await?);
                }
                &reqwest::StatusCode::CREATED => {
                    rec.push(res.json().await?);
                }
                _ => {
                    warn!("Response: {:?}", res.text().await?);
                }
            }
        }

        Ok(rec)
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
