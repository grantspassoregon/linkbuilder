use crate::authorize::{self, AuthorizedUser};
use crate::error;
use data_encoding::BASE64;
use indicatif::ProgressBar;
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::Read;
use tracing::{info, trace};

#[derive(Clone, Deserialize, Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Document {
    id: i32,
    name: String,
    description: Option<String>,
    status: Option<i32>,
    file_size: Option<f64>,
    created_date: Option<String>,
    created_by: Option<i32>,
    file_uploaded_date: Option<String>,
    file_uploaded_by: Option<i32>,
    is_visible: Option<bool>,
    start_date: Option<String>,
    end_date: Option<String>,
    file_type: Option<String>,
    #[serde(rename(deserialize = "URL"))]
    url: Option<String>,
    alt_text: Option<String>,
    folder_id: Option<i32>,
    convert_to_pdf: Option<bool>,
    file: Option<String>,
    file_name: Option<String>,
    is_archived: Option<bool>,
    is_chunked: Option<bool>,
    is_last_chunk: Option<bool>,
    upload_id: Option<String>,
    last_modified_by: Option<i32>,
    last_modified_on: Option<String>,
    show_archives: Option<bool>,
    show_in_rss_feed: Option<bool>,
}

impl Document {
    pub async fn upload(
        &self,
        url: &str,
        path: std::path::PathBuf,
        info: &DocInfo,
        user: &authorize::AuthorizedUser,
        publish: bool,
    ) -> Result<(), error::LinkError> {
        let mut status = "Draft".to_string();
        if publish {
            status = "Published".to_string();
        }
        let client = reqwest::Client::new();
        trace!("Upload client created.");
        let mut file = std::fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let enc = BASE64.encode(&data);
        let body = json!({
            "Name": self.name,
            "FileName": format!("{}.pdf", self.name),
            "File": format!("{}", enc),
            "FolderId": self.id,
            "Status": status,
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
        match &res.status() {
            &reqwest::StatusCode::OK => Ok(res.json().await?),
            &reqwest::StatusCode::CREATED => Ok(res.json().await?),
            _ => {
                info!("Response: {:?}", res.text().await?);
                Err(error::LinkError::AuthError)
            }
        }
    }

    pub async fn update(
        &self,
        url: &str,
        info: &DocInfo,
        user: &authorize::AuthorizedUser,
        command: &str,
    ) -> Result<String, error::LinkError> {
        trace!("Doc name: {}", self.name());
        trace!("Doc id: {}", self.id());
        trace!("Doc url: {:?}", self.url_ref());
        let mut doc = self.clone();

        match command {
            "archive" => {
                doc.is_archived = Some(true);
            }
            "draft" => {
                doc.status = Some(10);
            }
            _ => {}
        }

        let doc = serde_json::to_string(&doc)?;

        let client = reqwest::Client::new();
        trace!("Client created for update.");
        let endpoint = format!("{}/{}", url, self.id());

        let res = client
            .put(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(info.headers().api_key(), user.api_key())
            .header(info.headers().partition(), user.partition())
            .header(info.headers().user_api_key(), user.user_api_key())
            .body(doc)
            .send()
            .await?;
        match &res.status() {
            &reqwest::StatusCode::OK => Ok(res.json().await?),
            _ => {
                // info!("Response: {:?}", res.text().await?);
                // Err(error::LinkError::AuthError)
                Ok(res.text().await?)
            }
        }
    }

    pub async fn delete(
        &self,
        url: &str,
        info: &DocInfo,
        user: &authorize::AuthorizedUser,
    ) -> Result<String, error::LinkError> {
        let client = reqwest::Client::new();
        trace!("Client created for delete.");
        let endpoint = format!("{}/{}", url, self.id());
        let res = client
            .delete(endpoint)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(info.headers().api_key(), user.api_key())
            .header(info.headers().partition(), user.partition())
            .header(info.headers().user_api_key(), user.user_api_key())
            .send()
            .await?;
        match &res.status() {
            &reqwest::StatusCode::OK => Ok(res.json().await?),
            _ => {
                // info!("Response: {:?}", res.text().await?);
                // Err(error::LinkError::AuthError)
                Ok(res.text().await?)
            }
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn file_size(&self) -> Option<f64> {
        self.file_size.clone()
    }

    pub fn status_ref(&self) -> &Option<i32> {
        &self.status
    }

    pub fn url(&self) -> Option<String> {
        self.url.clone()
    }

    pub fn url_ref(&self) -> &Option<String> {
        &self.url
    }

    pub fn is_archived(&self) -> &Option<bool> {
        &self.is_archived
    }

    pub fn rss_feed_ref(&self) -> &Option<bool> {
        &self.show_in_rss_feed
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Documents {
    current_page: Option<i32>,
    page_size: Option<i32>,
    total_count: Option<i32>,
    total_pages: Option<i32>,
    source: Option<Vec<Document>>,
    sort_by: Option<String>,
    filter: Option<String>,
    has_previous_page: Option<bool>,
    has_next_page: Option<bool>,
}

impl Documents {
    pub async fn update(
        &self,
        url: &str,
        info: &DocInfo,
        user: &authorize::AuthorizedUser,
        command: &str,
    ) -> Result<Vec<String>, error::LinkError> {
        let mut res = Vec::new();
        if let Some(docs) = self.source_ref() {
            let style = indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {'Updating files.'}",
            )
            .unwrap();
            let bar = ProgressBar::new(docs.len() as u64);
            bar.set_style(style);
            for doc in docs {
                res.push(doc.update(url, info, user, command).await?);
                bar.inc(1);
            }
        }
        Ok(res)
    }

    pub async fn query(info: &DocInfo, user: &AuthorizedUser) -> Result<Self, error::LinkError> {
        let client = reqwest::Client::new();
        let res = client
            .get(info.query())
            .header(ACCEPT, "application/json")
            .header(info.headers.clone().api_key(), user.api_key())
            .header(info.headers.clone().partition(), user.partition())
            .header(info.headers.clone().user_api_key(), user.user_api_key())
            .send()
            .await?;
        match &res.status() {
            &reqwest::StatusCode::OK => Ok(res.json::<Documents>().await?),
            _ => Err(error::LinkError::AuthError),
        }
    }

    pub fn current_page_ref(&self) -> &Option<i32> {
        &self.current_page
    }

    pub fn page_size(&self) -> Option<i32> {
        self.page_size.clone()
    }

    pub fn total_count(&self) -> Option<i32> {
        self.total_count.clone()
    }

    pub fn total_pages_ref(&self) -> &Option<i32> {
        &self.total_pages
    }

    pub fn source(&self) -> Option<Vec<Document>> {
        self.source.clone()
    }

    pub fn source_ref(&self) -> &Option<Vec<Document>> {
        &self.source
    }

    pub fn sort_by_ref(&self) -> &Option<String> {
        &self.sort_by
    }

    pub fn filter_ref(&self) -> &Option<String> {
        &self.filter
    }

    pub fn has_previous_page_ref(&self) -> &Option<bool> {
        &self.has_previous_page
    }

    pub fn has_next_page_ref(&self) -> &Option<bool> {
        &self.has_next_page
    }

    pub fn total_size(&self) -> f64 {
        let mut size = 0.;
        if let Some(docs) = self.source() {
            for doc in docs {
                if let Some(value) = doc.file_size() {
                    size += value;
                }
            }
        }
        size
    }

    pub async fn delete(
        &self,
        url: &str,
        info: &DocInfo,
        user: &authorize::AuthorizedUser,
    ) -> Result<Vec<String>, error::LinkError> {
        let mut res = Vec::new();
        if let Some(docs) = self.source_ref() {
            let style = indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {'Deleting files.'}",
            )
            .unwrap();
            let bar = ProgressBar::new(docs.len() as u64);
            bar.set_style(style);
            for doc in docs {
                res.push(doc.delete(url, info, user).await?);
                bar.inc(1);
            }
        }
        Ok(res)
    }
}

#[derive(Clone)]
pub struct DocumentHeaders {
    api_key: HeaderName,
    partition: HeaderName,
    user_api_key: HeaderName,
}

impl DocumentHeaders {
    pub fn new(api_key: &'static str, partition: &'static str, user_api_key: &'static str) -> Self {
        let api_key_header = HeaderName::from_static(api_key);
        let partition_header = HeaderName::from_static(partition);
        let user_api_key_header = HeaderName::from_static(user_api_key);
        DocumentHeaders {
            api_key: api_key_header,
            partition: partition_header,
            user_api_key: user_api_key_header,
        }
    }

    pub fn api_key(self) -> HeaderName {
        self.api_key
    }

    pub fn partition(self) -> HeaderName {
        self.partition
    }

    pub fn user_api_key(self) -> HeaderName {
        self.user_api_key
    }
}

impl Default for DocumentHeaders {
    fn default() -> Self {
        let api_key = HeaderName::from_static("apikey");
        let partition = HeaderName::from_static("partition");
        let user_api_key = HeaderName::from_static("userapikey");
        DocumentHeaders {
            api_key,
            partition,
            user_api_key,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct DocQuery {
    top: Option<i32>,
    skip: Option<i32>,
    filter: Option<String>,
    orderby: Option<String>,
    inlinecount: Option<String>,
    expand: Option<String>,
}

impl DocQuery {
    pub fn new() -> Self {
        DocQuery::default()
    }

    pub fn top(&mut self, value: i32) -> &mut Self {
        self.top = Some(value);
        self
    }

    pub fn skip(&mut self, value: i32) -> &mut Self {
        self.skip = Some(value);
        self
    }

    pub fn filter(&mut self, value: &str) -> &mut Self {
        self.filter = Some(value.to_owned());
        self
    }

    pub fn orderby(&mut self, value: &str) -> &mut Self {
        self.orderby = Some(value.to_owned());
        self
    }

    pub fn inlinecount(&mut self, value: &str) -> &mut Self {
        self.inlinecount = Some(value.to_owned());
        self
    }

    pub fn expand(&mut self, value: &str) -> &mut Self {
        self.expand = Some(value.to_owned());
        self
    }

    pub fn query(&self) -> String {
        let mut args = Vec::new();
        if let Some(arg) = self.top {
            args.push(format!("%24top={}", arg));
        }
        if let Some(arg) = self.skip {
            args.push(format!("%24skip={}", arg));
        }
        if let Some(arg) = self.filter.clone() {
            args.push(format!("%24filter={}", arg));
        }
        if let Some(arg) = self.orderby.clone() {
            args.push(format!("%24orderby={}", arg));
        }
        if let Some(arg) = self.inlinecount.clone() {
            args.push(format!("%24inlinecount={}", arg));
        }
        if let Some(arg) = self.expand.clone() {
            args.push(format!("%24expand={}", arg));
        }
        let mut query = "?".to_string();
        let mut i = 0;
        for arg in args {
            if i == 0 {
                query.push_str(&arg);
            } else {
                query.push_str(&format!("&{}", arg));
            }
            i += 1;
        }
        query
    }
}

pub struct DocInfo {
    headers: DocumentHeaders,
    query: DocQuery,
    url: String,
}

impl DocInfo {
    pub fn new(headers: &DocumentHeaders, query: &DocQuery, url: &str) -> Self {
        DocInfo {
            headers: headers.clone(),
            query: query.clone(),
            url: url.to_owned(),
        }
    }

    pub fn headers(&self) -> DocumentHeaders {
        self.headers.clone()
    }

    pub fn query(&self) -> String {
        format!("{}{}", self.url, self.query.query())
    }
}

#[derive(Debug)]
pub struct DocumentLinks {
    links: HashMap<String, std::path::PathBuf>,
}

impl DocumentLinks {
    pub fn new(links: HashMap<String, std::path::PathBuf>) -> Self {
        DocumentLinks { links }
    }

    pub fn ref_links(&self) -> &HashMap<String, std::path::PathBuf> {
        &self.links
    }
}

impl Default for DocumentLinks {
    fn default() -> Self {
        let links = HashMap::<String, std::path::PathBuf>::new();
        DocumentLinks { links }
    }
}

impl From<&Documents> for DocumentLinks {
    fn from(docs: &Documents) -> Self {
        let mut links = HashMap::<String, std::path::PathBuf>::new();
        if let Some(documents) = docs.source_ref() {
            for doc in documents {
                if let Some(url) = doc.url_ref() {
                    links.insert(doc.name(), url.into());
                }
            }
        }
        DocumentLinks::new(links)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Folders {
    current_page: Option<i32>,
    page_size: Option<i32>,
    total_count: Option<i32>,
    total_pages: Option<i32>,
    source: Option<Vec<Folder>>,
    sort_by: Option<String>,
    filter: Option<String>,
    has_previous_page: Option<bool>,
    has_next_page: Option<bool>,
}

impl Folders {
    pub async fn query(info: &DocInfo, user: &AuthorizedUser) -> Result<Self, error::LinkError> {
        let client = reqwest::Client::new();
        let res = client
            .get(info.query())
            .header(ACCEPT, "application/json")
            .header(info.headers().api_key(), user.api_key())
            .header(info.headers().partition(), user.partition())
            .header(info.headers().user_api_key(), user.user_api_key())
            .send()
            .await?;
        match &res.status() {
            &reqwest::StatusCode::OK => Ok(res.json::<Folders>().await?),
            _ => Err(error::LinkError::AuthError),
        }
    }

    pub fn current_page_ref(&self) -> &Option<i32> {
        &self.current_page
    }

    pub fn page_size(&self) -> Option<i32> {
        self.page_size.clone()
    }

    pub fn total_count(&self) -> Option<i32> {
        self.total_count.clone()
    }

    pub fn total_pages_ref(&self) -> &Option<i32> {
        &self.total_pages
    }

    pub fn source(&self) -> Option<Vec<Folder>> {
        self.source.clone()
    }

    pub fn sort_by_ref(&self) -> &Option<String> {
        &self.sort_by
    }

    pub fn filter_ref(&self) -> &Option<String> {
        &self.filter
    }

    pub fn has_previous_page_ref(&self) -> &Option<bool> {
        &self.has_previous_page
    }

    pub fn has_next_page_ref(&self) -> &Option<bool> {
        &self.has_next_page
    }

    pub fn get_id(&self, name: &str) -> Option<i32> {
        let mut id = None;
        if let Some(folders) = self.source() {
            for folder in folders {
                if name == "Fee in Lieu" {
                    id = Some(1884);
                }
                if folder.name == name && folder.is_archived_ref() == &Some(false) {
                    id = folder.id;
                }
            }
        }
        id
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Folder {
    id: Option<i32>,
    description: Option<String>,
    status: Option<i32>,
    path: Option<String>,
    name: String,
    #[serde(rename(deserialize = "ParentID"))]
    parent_id: Option<i32>,
    created_date: Option<String>,
    created_by: Option<i32>,
    last_modified_date: Option<String>,
    modified_by: Option<i32>,
    is_archived: Option<bool>,
    show_archive: Option<bool>,
    last_archived_date: Option<String>,
    update_integration_hub: Option<bool>,
    archived_by: Option<i32>,
    archived_reason: Option<i32>,
    #[serde(rename(deserialize = "ArchivedFolderID"))]
    archived_folder_id: Option<i32>,
    total_folder_size: Option<i32>,
    children_exist: Option<bool>,
    #[serde(rename(deserialize = "URL"))]
    url: Option<String>,
    folder_root: Option<i32>,
    department_header_id: Option<i32>,
    show_archives: Option<bool>,
    permissions: Option<Vec<String>>,
    item_count: Option<i32>,
}

impl Folder {
    pub fn id_ref(&self) -> &Option<i32> {
        &self.id
    }

    pub fn description_ref(&self) -> &Option<String> {
        &self.description
    }

    pub fn status_ref(&self) -> &Option<i32> {
        &self.status
    }

    pub fn path_ref(&self) -> &Option<String> {
        &self.path
    }

    pub fn parent_id_ref(&self) -> &Option<i32> {
        &self.parent_id
    }

    pub fn created_date_ref(&self) -> &Option<String> {
        &self.created_date
    }

    pub fn created_by_ref(&self) -> &Option<i32> {
        &self.created_by
    }

    pub fn last_modified_date_ref(&self) -> &Option<String> {
        &self.last_modified_date
    }

    pub fn modified_by_ref(&self) -> &Option<i32> {
        &self.modified_by
    }

    pub fn is_archived_ref(&self) -> &Option<bool> {
        &self.is_archived
    }

    pub fn show_archive_ref(&self) -> &Option<bool> {
        &self.show_archive
    }

    pub fn last_archived_date_ref(&self) -> &Option<String> {
        &self.last_archived_date
    }

    pub fn update_integration_hub_ref(&self) -> &Option<bool> {
        &self.update_integration_hub
    }

    pub fn archived_by_ref(&self) -> &Option<i32> {
        &self.archived_by
    }

    pub fn archived_reason_ref(&self) -> &Option<i32> {
        &self.archived_reason
    }

    pub fn archived_folder_id_ref(&self) -> &Option<i32> {
        &self.archived_folder_id
    }

    pub fn total_folder_size_ref(&self) -> &Option<i32> {
        &self.total_folder_size
    }

    pub fn children_exist_ref(&self) -> &Option<bool> {
        &self.children_exist
    }

    pub fn url_ref(&self) -> &Option<String> {
        &self.url
    }

    pub fn folder_root_ref(&self) -> &Option<i32> {
        &self.folder_root
    }

    pub fn department_header_id_ref(&self) -> &Option<i32> {
        &self.department_header_id
    }

    pub fn show_archives_ref(&self) -> &Option<bool> {
        &self.show_archives
    }

    pub fn permissions_ref(&self) -> &Option<Vec<String>> {
        &self.permissions
    }

    pub fn item_count_ref(&self) -> &Option<i32> {
        &self.item_count
    }
}
