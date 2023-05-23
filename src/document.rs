use crate::authorize::AuthorizedUser;
use crate::error;
use reqwest::header::{HeaderName, ACCEPT};
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Document {
    id: i32,
    name: String,
    description: Option<String>,
    status: Option<i32>,
    file_size: Option<f32>,
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

    pub fn page_size(&self) -> Option<i32> {
        self.page_size.clone()
    }

    pub fn total_count(&self) -> Option<i32> {
        self.total_count.clone()
    }

    pub fn source(&self) -> Option<Vec<Document>> {
        self.source.clone()
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

impl Folders {
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
            &reqwest::StatusCode::OK => Ok(res.json::<Folders>().await?),
            _ => Err(error::LinkError::AuthError),
        }
    }

    pub fn page_size(&self) -> Option<i32> {
        self.page_size.clone()
    }

    pub fn total_count(&self) -> Option<i32> {
        self.total_count.clone()
    }

    pub fn source(&self) -> Option<Vec<Folder>> {
        self.source.clone()
    }
}
