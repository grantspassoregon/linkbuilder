//! Data structures and methods for CivicEngage API responses.
//!
//! # Examples
//!
//! ```rust
//! # use linkbuilder::prelude::{AuthorizeHeaders, AuthorizeInfo, AuthorizedUser, DocInfo, DocumentHeaders, Documents, DocQuery, FileNames, Folders, LinkError, LinkResult, User};
//! # #[tokio::main]
//! # async fn main() -> LinkResult<()> {
//! dotenv::dotenv().ok();
//! let api_key = std::env::var("API_KEY")?;
//! let partition = std::env::var("PARTITION")?;
//! let name = std::env::var("USERNAME")?;
//! let password = std::env::var("PASSWORD")?;
//! let host = std::env::var("HOST")?;
//!
//! // Authorization
//! let user = User::new()
//!     .api_key(&api_key)
//!     .partition(&partition)
//!     .name(&name)
//!     .password(&password)
//!     .host(&host)
//!     .build()?;
//! let headers = AuthorizeHeaders::default();
//! let auth_info = AuthorizeInfo::new(&user, headers);
//! let url = std::env::var("AUTHENTICATE")?;
//! let response = auth_info.authorize(&url).await?;
//! let auth_user = AuthorizedUser::new(&user, &response);
//!
//! // Upload
//! let headers = DocumentHeaders::default();
//! let mut args = DocQuery::new();
//! args.inlinecount("allpages");
//! let url = std::env::var("FOLDER")?;
//! let doc_info = DocInfo::new(&headers, &args, &url);
//! let folders = Folders::query(&doc_info, &auth_user).await?;
//! let file = FileNames::from_path("c:/users/erose/repos/linkbuilder/public")?;
//! if let Some(id) = folders.get_id("test") {
//!     file.upload(&doc_info, &auth_user, id).await?;
//! }
//!
//! // Update
//! if let Some(id) = folders.get_id("test") {
//!     args.filter(&format!("FolderId eq {}", id));
//!     let url = std::env::var("DOCUMENT")?;
//!     let doc_info = DocInfo::new(&headers, &args, &url);
//!     let docs = Documents::query(&doc_info, &auth_user).await?;
//!     let response = docs.update(&doc_info, &auth_user, "draft").await?;
//! // Delete
//!     let response = docs.delete(&doc_info, &auth_user).await?;
//! }
//! # Ok(())
//! # }
use crate::prelude::*;
use data_encoding::BASE64;
use indicatif::ProgressBar;
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::io::Read;
use tracing::{info, trace, warn};

/// Data type for Document responses from the Document Center on CivicEngage.
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
    /// Upload a document to Document Center on CivicEngage.
    pub async fn upload(
        &self,
        path: std::path::PathBuf,
        info: &DocInfo,
        user: &AuthorizedUser,
        publish: bool,
    ) -> LinkResult<()> {
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
            .post(info.url_ref())
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
                Err(LinkError::AuthError)
            }
        }
    }

    /// Update document in Document Center on CivicEngage. Called by [`Documents::update()`].
    /// The `command` field takes a string of value "draft" or "archive", and will set the status
    /// of the document to "Draft" or "Archived" respectively.
    pub async fn update(
        &self,
        info: &DocInfo,
        user: &AuthorizedUser,
        command: &str,
    ) -> LinkResult<String> {
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
        let endpoint = format!("{}/{}", info.url_ref(), self.id());

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
            _ => Ok(res.text().await?),
        }
    }

    /// Delete document from Document Center on CivicEngage.  Called by [`Documents::delete()`].
    pub async fn delete(&self, info: &DocInfo, user: &AuthorizedUser) -> LinkResult<String> {
        let client = reqwest::Client::new();
        trace!("Client created for delete.");
        let endpoint = format!("{}/{}", info.url_ref(), self.id());
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

    /// The `id` field represents the folder id on the Document Center.  This function returns the
    /// value of the field.
    pub fn id(&self) -> i32 {
        self.id
    }

    /// The `name` field represents the document name on the Document Center.  This function returns the
    /// cloned value of the field.
    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    /// The `file_size` field represents the document file size on the Document Center in KB.  This function returns the
    /// cloned value of the field.
    pub fn file_size(&self) -> Option<f64> {
        self.file_size.clone()
    }

    /// The `status_ref` field represents the document status on the Document Center.  This function returns the
    /// cloned value of the field. The status is integer coded, with `10` corresponding to "Draft"
    /// status and `30` corresponding to "Published".
    pub fn status_ref(&self) -> &Option<i32> {
        &self.status
    }

    /// The `url` field represents the document file url on the Document Center in KB.  This function returns the
    /// cloned value of the field.
    pub fn url(&self) -> Option<String> {
        self.url.clone()
    }

    /// This function returns a reference to the value of the `url` field.
    pub fn url_ref(&self) -> &Option<String> {
        &self.url
    }

    /// The `is_archived` field represents whether the document on the Document Center is archived.  This function returns a reference to the
    /// value of the field.
    pub fn is_archived(&self) -> &Option<bool> {
        &self.is_archived
    }

    /// The `rss_feed_ref` field represents whether the document on the Document Center is archived.  This function returns a reference to the
    /// value of the field.
    pub fn rss_feed_ref(&self) -> &Option<bool> {
        &self.show_in_rss_feed
    }
}

/// Holds a Documents response from the Document Center on CivicEngage.
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
    /// Update all documents in [`Documents`].  The `command` field takes a string of value "draft"
    /// or "archive", updating the status of documents to "Draft" or "Archived" respectively.
    /// Documents with status "Published" cannot be deleted and must be set to "Draft" first.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::{AuthorizeHeaders, AuthorizeInfo, AuthorizedUser, DocInfo, DocumentHeaders, Documents, DocQuery, FileNames, Folders, LinkError, LinkResult, User};
    /// # #[tokio::main]
    /// # async fn main() -> LinkResult<()> {
    /// dotenv::dotenv().ok();
    /// let api_key = std::env::var("API_KEY")?;
    /// let partition = std::env::var("PARTITION")?;
    /// let name = std::env::var("USERNAME")?;
    /// let password = std::env::var("PASSWORD")?;
    /// let host = std::env::var("HOST")?;
    ///
    /// // Authorization
    /// let user = User::new()
    ///     .api_key(&api_key)
    ///     .partition(&partition)
    ///     .name(&name)
    ///     .password(&password)
    ///     .host(&host)
    ///     .build()?;
    /// let headers = AuthorizeHeaders::default();
    /// let auth_info = AuthorizeInfo::new(&user, headers);
    /// let url = std::env::var("AUTHENTICATE")?;
    /// let response = auth_info.authorize(&url).await?;
    /// let auth_user = AuthorizedUser::new(&user, &response);
    ///
    /// // Upload
    /// let headers = DocumentHeaders::default();
    /// let mut args = DocQuery::new();
    /// args.inlinecount("allpages");
    /// let url = std::env::var("FOLDER")?;
    /// let doc_info = DocInfo::new(&headers, &args, &url);
    /// let folders = Folders::query(&doc_info, &auth_user).await?;
    /// let file = FileNames::from_path("c:/users/erose/repos/linkbuilder/public")?;
    /// if let Some(id) = folders.get_id("test") {
    ///     file.upload(&doc_info, &auth_user, id).await?;
    /// }
    ///
    /// // Update
    /// if let Some(id) = folders.get_id("test") {
    ///     args.filter(&format!("FolderId eq {}", id));
    ///     let url = std::env::var("DOCUMENT")?;
    ///     let doc_info = DocInfo::new(&headers, &args, &url);
    ///     let docs = Documents::query(&doc_info, &auth_user).await?;
    ///     let response = docs.update(&doc_info, &auth_user, "draft").await?;
    /// # // Delete
    /// #     let response = docs.delete(&doc_info, &auth_user).await?;
    /// }
    /// # Ok(())
    /// # }
    pub async fn update(
        &self,
        info: &DocInfo,
        user: &AuthorizedUser,
        command: &str,
    ) -> LinkResult<Vec<String>> {
        let mut res = Vec::new();
        if let Some(docs) = self.source_ref() {
            let style = indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {'Updating files.'}",
            )
            .unwrap();
            let bar = ProgressBar::new(docs.len() as u64);
            bar.set_style(style);
            for doc in docs {
                res.push(doc.update(info, user, command).await?);
                bar.inc(1);
            }
        }
        Ok(res)
    }

    /// Sends a search request to the Document Center using the query parameters from `info`.
    /// Calls [`DocInfo::query()`], which calls [`DocQuery::query()`].
    pub async fn query(info: &DocInfo, user: &AuthorizedUser) -> LinkResult<Self> {
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
            _ => Err(LinkError::AuthError),
        }
    }

    /// The `current_page` field represents the page number of the paginated list.  This function returns a reference
    /// to the field.
    pub fn current_page_ref(&self) -> &Option<i32> {
        &self.current_page
    }

    /// The `page_size` field represents the size of the current page.  This function returns the cloned value
    /// of the field.
    pub fn page_size(&self) -> Option<i32> {
        self.page_size.clone()
    }

    /// The `total_count` field represents the total count of items in the paginated list.  This function returns the cloned value
    /// of the field.
    pub fn total_count(&self) -> Option<i32> {
        self.total_count.clone()
    }

    /// The `total_pages` field represents the page count of the paginated list.  This function returns a reference
    /// to the field.
    pub fn total_pages_ref(&self) -> &Option<i32> {
        &self.total_pages
    }

    /// The `source` field represents the source of the paginated list.  This function returns the cloned value
    /// of the field.
    pub fn source(&self) -> Option<Vec<Document>> {
        self.source.clone()
    }

    /// This functions returns a reference to the value in the `source` field.
    pub fn source_ref(&self) -> &Option<Vec<Document>> {
        &self.source
    }

    /// The `sort_by` field represents the sorting order of the paginated list.  This function returns a reference
    /// to the field.
    pub fn sort_by_ref(&self) -> &Option<String> {
        &self.sort_by
    }

    /// The `filter` field represents the filters applied to the paginated list.  This function returns a reference
    /// to the field.
    pub fn filter_ref(&self) -> &Option<String> {
        &self.filter
    }

    /// The `has_previous_page` field indicates whether the paginated list contains a previous
    /// page.  This function returns a reference to the field.
    pub fn has_previous_page_ref(&self) -> &Option<bool> {
        &self.has_previous_page
    }

    /// The `has_next_page` field indicates whether the paginated list contains a next
    /// page.  This function returns a reference to the field.
    pub fn has_next_page_ref(&self) -> &Option<bool> {
        &self.has_next_page
    }

    /// Loops through documents in `Documents` to calculate the total size in KB.
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

    /// Delete all documents in `Documents` from the Document Center on CivicEngage.  Calls
    /// [`Document::delete()`].
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::{AuthorizeHeaders, AuthorizeInfo, AuthorizedUser, DocInfo, DocumentHeaders, Documents, DocQuery, FileNames, Folders, LinkError, LinkResult, User};
    /// # #[tokio::main]
    /// # async fn main() -> LinkResult<()> {
    /// dotenv::dotenv().ok();
    /// let api_key = std::env::var("API_KEY")?;
    /// let partition = std::env::var("PARTITION")?;
    /// let name = std::env::var("USERNAME")?;
    /// let password = std::env::var("PASSWORD")?;
    /// let host = std::env::var("HOST")?;
    ///
    /// // Authorization
    /// let user = User::new()
    ///     .api_key(&api_key)
    ///     .partition(&partition)
    ///     .name(&name)
    ///     .password(&password)
    ///     .host(&host)
    ///     .build()?;
    /// let headers = AuthorizeHeaders::default();
    /// let auth_info = AuthorizeInfo::new(&user, headers);
    /// let url = std::env::var("AUTHENTICATE")?;
    /// let response = auth_info.authorize(&url).await?;
    /// let auth_user = AuthorizedUser::new(&user, &response);
    ///
    /// // Upload
    /// let headers = DocumentHeaders::default();
    /// let mut args = DocQuery::new();
    /// args.inlinecount("allpages");
    /// let url = std::env::var("FOLDER")?;
    /// let doc_info = DocInfo::new(&headers, &args, &url);
    /// let folders = Folders::query(&doc_info, &auth_user).await?;
    /// let file = FileNames::from_path("c:/users/erose/repos/linkbuilder/public")?;
    /// if let Some(id) = folders.get_id("test") {
    ///     file.upload(&doc_info, &auth_user, id).await?;
    /// }
    ///
    /// // Update
    /// if let Some(id) = folders.get_id("test") {
    ///     args.filter(&format!("FolderId eq {}", id));
    ///     let url = std::env::var("DOCUMENT")?;
    ///     let doc_info = DocInfo::new(&headers, &args, &url);
    ///     let docs = Documents::query(&doc_info, &auth_user).await?;
    ///     let response = docs.update(&doc_info, &auth_user, "draft").await?;
    /// // Delete
    ///     let response = docs.delete(&doc_info, &auth_user).await?;
    /// }
    /// # Ok(())
    /// # }
    pub async fn delete(&self, info: &DocInfo, user: &AuthorizedUser) -> LinkResult<Vec<String>> {
        let mut res = Vec::new();
        if let Some(docs) = self.source_ref() {
            let style = indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {'Deleting files.'}",
            )
            .unwrap();
            let bar = ProgressBar::new(docs.len() as u64);
            bar.set_style(style);
            for doc in docs {
                res.push(doc.delete(info, user).await?);
                bar.inc(1);
            }
        }
        Ok(res)
    }
}

/// Holds headers for calls to the Document endpoint on CivicEngage.
#[derive(Clone, Debug)]
pub struct DocumentHeaders {
    api_key: HeaderName,
    partition: HeaderName,
    user_api_key: HeaderName,
}

impl DocumentHeaders {
    /// Creates a new `DocumentHeaders` from string values for the `api_key`, `partition` and
    /// `user_api_key` fields.
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

    /// The `api_key` field contains the `HeaderName` for the user API key.  This function returns the cloned
    /// value of the field.
    pub fn api_key(self) -> HeaderName {
        self.api_key
    }

    /// The `partition` field contains the `HeaderName` for the user partition number.  This function returns the cloned
    /// value of the field.
    pub fn partition(self) -> HeaderName {
        self.partition
    }

    /// The `user_api_key` field contains the `HeaderName` for the user session id.  This function returns the cloned
    /// value of the field.
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

/// Holds query parameters for searching the Document Center.  Sets and stores these query
/// parameters, and converts them to a url-encoded string for use in search.
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
    /// Creates a new `DocQuery` struct with all field values initiated to `None`.
    pub fn new() -> Self {
        DocQuery::default()
    }

    /// The `top` field represents the number of pages from a paginated list to return in a search,
    /// beginning with the first page.  This function returns a mutable reference
    /// to the field.
    pub fn top(&mut self, value: i32) -> &mut Self {
        self.top = Some(value);
        self
    }

    /// The `skip` field represents the number of records from a paginated list to skip in a search.  
    /// This function returns a mutable reference to the field.
    pub fn skip(&mut self, value: i32) -> &mut Self {
        self.skip = Some(value);
        self
    }

    /// The `filter` field contains an equation that must evaluate to true to return a record.  
    /// This function returns a mutable reference to the field.
    pub fn filter(&mut self, value: &str) -> &mut Self {
        self.filter = Some(value.to_owned());
        self
    }

    /// The `orderby` field contains the values used to order a collection of records.  
    /// This function returns a mutable reference to the field.
    pub fn orderby(&mut self, value: &str) -> &mut Self {
        self.orderby = Some(value.to_owned());
        self
    }

    /// The `inlinecount` field will include pagination details if set to "allpages".  
    /// This function returns a mutable reference to the field.
    pub fn inlinecount(&mut self, value: &str) -> &mut Self {
        self.inlinecount = Some(value.to_owned());
        self
    }

    /// The `expand` field will include additional details for the categories "Permissions",
    /// "Images" and "Links".  This function returns a mutable reference to the field.
    pub fn expand(&mut self, value: &str) -> &mut Self {
        self.expand = Some(value.to_owned());
        self
    }

    /// Constructs a url-encoded search string from the search parameters stored in the struct.
    /// Appended to the base url in [`DocInfo`] to create the API endpoint.
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

/// The `DocInfo` struct holds the parameters for forming a query request to the Document Center on
/// CivicEngage, including headers, query parameters and the target url.
pub struct DocInfo {
    headers: DocumentHeaders,
    query: DocQuery,
    url: String,
}

impl DocInfo {
    /// Creates a new `DocInfo` instance from references to a [`DocumentHeaders`] struct, a
    /// [`DocQuery`] struct, and the target url.
    pub fn new(headers: &DocumentHeaders, query: &DocQuery, url: &str) -> Self {
        DocInfo {
            headers: headers.clone(),
            query: query.clone(),
            url: url.to_owned(),
        }
    }

    /// The `headers` field contains the [`DocumentHeaders`].  This function returns the cloned
    /// value of the field.
    pub fn headers(&self) -> DocumentHeaders {
        self.headers.clone()
    }

    /// The `url` field contains the url endpoint target of the request.
    pub fn url_ref(&self) -> &String {
        &self.url
    }

    /// Returns the target url with the url-encoded query parameters appended to the endpoint.
    /// Calls [`DocQuery::query()`].
    pub fn query(&self) -> String {
        format!("{}{}", self.url, self.query.query())
    }
}

/// Holds a HashMap of file names and file paths, used to gather active links from files stored in
/// the Document Center for tranfer to the GIS layers.
#[derive(Debug)]
pub struct DocumentLinks {
    links: HashMap<String, std::path::PathBuf>,
}

impl DocumentLinks {
    /// Creates a new `DocumentLinks` from a HashMap of file names and file urls.
    pub fn new(links: HashMap<String, std::path::PathBuf>) -> Self {
        DocumentLinks { links }
    }

    /// The `links` field contains a HashMap of files names and file paths.  This function returns a reference
    /// to the field.
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

/// Data type for a paginated list of type [`Folder`] response from the Document Center on CivicEngage.
#[derive(Deserialize, Debug, Default, Clone)]
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
    /// Submits a query request based upon parameters set in `info`.  Calls [`DocInfo::query()`].
    pub async fn query(info: &DocInfo, user: &AuthorizedUser) -> LinkResult<Self> {
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
            _ => Err(LinkError::AuthError),
        }
    }

    /// The `current_page` field represents the page number of the paginated list.  This function returns a reference
    /// to the field.
    pub fn current_page_ref(&self) -> &Option<i32> {
        &self.current_page
    }

    /// The `page_size` field represents the size of the current page.  This function returns the cloned value
    /// of the field.
    pub fn page_size(&self) -> Option<i32> {
        self.page_size.clone()
    }

    /// The `total_count` field represents the total count of items in the paginated list.  This function returns the cloned value
    /// of the field.
    pub fn total_count(&self) -> Option<i32> {
        self.total_count.clone()
    }

    /// The `total_pages` field represents the page count of the paginated list.  This function returns a reference
    /// to the field.
    pub fn total_pages_ref(&self) -> &Option<i32> {
        &self.total_pages
    }

    /// The `source` field represents the source of the paginated list.  This function returns the cloned value
    /// of the field.
    pub fn source(&self) -> Option<Vec<Folder>> {
        self.source.clone()
    }

    /// The `sort_by` field represents the sorting order of the paginated list.  This function returns a reference
    /// to the field.
    pub fn sort_by_ref(&self) -> &Option<String> {
        &self.sort_by
    }

    /// The `filter` field represents the filters applied to the paginated list.  This function returns a reference
    /// to the field.
    pub fn filter_ref(&self) -> &Option<String> {
        &self.filter
    }

    /// The `has_previous_page` field indicates whether the paginated list contains a previous
    /// page.  This function returns a reference to the field.
    pub fn has_previous_page_ref(&self) -> &Option<bool> {
        &self.has_previous_page
    }

    /// The `has_next_page` field indicates whether the paginated list contains a next
    /// page.  This function returns a reference to the field.
    pub fn has_next_page_ref(&self) -> &Option<bool> {
        &self.has_next_page
    }

    /// Searches for a folder in `Folders` where the folder name matches `name`.  Returns the
    /// folder id if present, and `None` if absent.  Folders that have been archived have a
    /// separate id associated with the active and archived versions, and this functions returns
    /// the active id.
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

/// Data type for Folder responses from the Document Center on CivicEngage.
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
    /// The `id` field represents the folder id of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn id_ref(&self) -> &Option<i32> {
        &self.id
    }

    /// The `description` field represents the description of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn description_ref(&self) -> &Option<String> {
        &self.description
    }

    /// The `status` field represents the integer-coded status of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn status_ref(&self) -> &Option<i32> {
        &self.status
    }

    /// The `path` field represents the directory path of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn path_ref(&self) -> &Option<String> {
        &self.path
    }

    /// The `parent_id` field represents the folder id for the parent of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn parent_id_ref(&self) -> &Option<i32> {
        &self.parent_id
    }

    /// The `created_date` field represents the creation date of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn created_date_ref(&self) -> &Option<String> {
        &self.created_date
    }

    /// The `created_by` field represents the user id for the creator of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn created_by_ref(&self) -> &Option<i32> {
        &self.created_by
    }

    /// The `last_modified_date` field represents the date that a `Folder` was last edited.  This function returns a reference
    /// to the field.
    pub fn last_modified_date_ref(&self) -> &Option<String> {
        &self.last_modified_date
    }

    /// The `modified_by` field represents the user id of the person who last modified the `Folder`.  This function returns a reference
    /// to the field.
    pub fn modified_by_ref(&self) -> &Option<i32> {
        &self.modified_by
    }

    /// The `is_archived` field indicates whether the `Folder` is archived.  This function returns a reference
    /// to the field.
    pub fn is_archived_ref(&self) -> &Option<bool> {
        &self.is_archived
    }

    /// The `show_archive` field indicates whether archived documents in the `Folder` should be shown.  This function returns a reference
    /// to the field.
    pub fn show_archive_ref(&self) -> &Option<bool> {
        &self.show_archive
    }

    /// The `last_archived_date` field represents the archive date of a `Folder`.  This function returns a reference
    /// to the field.
    pub fn last_archived_date_ref(&self) -> &Option<String> {
        &self.last_archived_date
    }

    /// The `update_integration_hub` field indicates whether to update the integration hub.  This function returns a reference
    /// to the field.
    pub fn update_integration_hub_ref(&self) -> &Option<bool> {
        &self.update_integration_hub
    }

    /// The `archived_by` field represents the user id of the person to archive the [`Folder`].  This function returns a reference
    /// to the field.
    pub fn archived_by_ref(&self) -> &Option<i32> {
        &self.archived_by
    }

    /// The `archived_reason` field is an integer code representing the reason for archiving the
    /// [`Folder`].  This function returns a reference to the field.
    pub fn archived_reason_ref(&self) -> &Option<i32> {
        &self.archived_reason
    }

    /// The `archived_folder_id` field represents the folder id of the archived [`Folder`].  This function returns a reference
    /// to the field.
    pub fn archived_folder_id_ref(&self) -> &Option<i32> {
        &self.archived_folder_id
    }

    /// The `total_folder_size` field represents the size of a [`Folder`] in bytes.  This function returns a reference
    /// to the field.
    pub fn total_folder_size_ref(&self) -> &Option<i32> {
        &self.total_folder_size
    }

    /// The `children_exist` field indicates whether the [`Folder`] contains children folders.  This function returns a reference
    /// to the field.
    pub fn children_exist_ref(&self) -> &Option<bool> {
        &self.children_exist
    }

    /// The `url_ref` field represents the url path to a [`Folder`].  This function returns a reference
    /// to the field.
    pub fn url_ref(&self) -> &Option<String> {
        &self.url
    }

    /// The `folder_root` field represents the [`Folder`] root, taking one of the value ['DocumentCenter', 'Content', 'Design', 'Banners', 'CivicSend', 'MyAccount'].
    /// This function returns a reference to the field.
    pub fn folder_root_ref(&self) -> &Option<i32> {
        &self.folder_root
    }

    /// The `document_header_id` field represents the department header saved for this [`Folder`].  This function returns a reference
    /// to the field.
    pub fn department_header_id_ref(&self) -> &Option<i32> {
        &self.department_header_id
    }

    /// The `show_archives` field gets or sets whether the [`Folder`] should show archives
    /// publically.  This function returns a reference to the field.
    pub fn show_archives_ref(&self) -> &Option<bool> {
        &self.show_archives
    }

    /// The `permissions` field represents the view/update/delete permissions associated with the [`Folder`].  This function returns a reference
    /// to the field.
    pub fn permissions_ref(&self) -> &Option<Vec<String>> {
        &self.permissions
    }

    /// The `item_count` field represents the total number of items in the [`Folder`].  This function returns a reference
    /// to the field.
    pub fn item_count_ref(&self) -> &Option<i32> {
        &self.item_count
    }
}

/// The `LinkUpdaterBuilder` struct is a builder struct for the [`LinkUpdater`], allowing the user
/// to construct the object incrementally.
#[derive(Default)]
pub struct LinkUpdaterBuilder {
    folders: Option<Folders>,
    headers: Option<DocumentHeaders>,
    args: Option<DocQuery>,
    url: Option<String>,
    user: Option<AuthorizedUser>,
    output: Option<String>,
}

impl LinkUpdaterBuilder {
    /// The `new` function creates an empty `LinkUpdaterBuilder` struct with all fields set to
    /// None.
    pub fn new() -> LinkUpdaterBuilder {
        LinkUpdaterBuilder::default()
    }

    /// The `folders()` function sets the value of the `folders` field to `value`.
    pub fn folders(&mut self, value: &Folders) -> &mut Self {
        self.folders = Some(value.clone());
        self
    }

    /// The `headers()` function sets the value of the `headers` field to `value`.
    pub fn headers(&mut self, value: &DocumentHeaders) -> &mut Self {
        self.headers = Some(value.clone());
        self
    }

    /// The `args()` function sets the value of the `args` field to `value`.
    pub fn args(&mut self, value: &DocQuery) -> &mut Self {
        self.args = Some(value.clone());
        self
    }

    /// The `url()` function sets the value of the `url` field to `value`.
    pub fn url(&mut self, value: &str) -> &mut Self {
        self.url = Some(value.into());
        self
    }

    /// The `user()` function sets the value of the `user` field to `value`.
    pub fn user(&mut self, value: &AuthorizedUser) -> &mut Self {
        self.user = Some(value.clone());
        self
    }

    /// The `output()` function sets the value of the `output` field to `value`.
    pub fn output(&mut self, value: &Option<String>) -> LinkResult<&mut Self> {
        match value {
            Some(_) => {
                self.output = value.clone();
                Ok(self)
            }
            None => {
                warn!("Missing output parameter.");
                Err(LinkError::BuildError)
            }
        }
    }

    /// The `build()` function returns a complete [`LinkUpdater`] struct if all the fields have
    /// been set.
    pub fn build(&self) -> LinkResult<LinkUpdater> {
        if let Some(folders) = self.folders.clone() {
            if let Some(headers) = self.headers.clone() {
                if let Some(args) = self.args.clone() {
                    if let Some(url) = self.url.clone() {
                        if let Some(user) = self.user.clone() {
                            if let Some(output) = self.output.clone() {
                                Ok(LinkUpdater {
                                    folders,
                                    headers,
                                    args,
                                    url,
                                    user,
                                    output,
                                })
                            } else {
                                Err(LinkError::BuildError)
                            }
                        } else {
                            Err(LinkError::BuildError)
                        }
                    } else {
                        Err(LinkError::BuildError)
                    }
                } else {
                    Err(LinkError::BuildError)
                }
            } else {
                Err(LinkError::BuildError)
            }
        } else {
            Err(LinkError::BuildError)
        }
    }
}

/// The `LinkUpdater` struct holds required fields for updating link files.
#[derive(Clone, Default, Debug)]
pub struct LinkUpdater {
    folders: Folders,
    headers: DocumentHeaders,
    args: DocQuery,
    url: String,
    user: AuthorizedUser,
    output: String,
}

impl LinkUpdater {
    /// The `new()` method creates an empty [`LinkUpdaterBuilder`].  Set the empty fields and
    /// call [`LinkUpdaterBuilder::build()`] to create a new `LinkUpdater`.
    pub fn new() -> LinkUpdaterBuilder {
        LinkUpdaterBuilder::new()
    }

    /// The `get_links()` method searches for links in folder `folder` and outputs a link file.
    pub async fn get_links(&self, folder: &str, file: &str) -> LinkResult<()> {
        if let Some(id) = self.folders.get_id(folder) {
            trace!("Folder id: {:?}", id);
            trace!("Specify folder for search.");
            let mut args = self.args.clone();
            args.filter(&format!("FolderId eq {}", id));
            let doc_info = DocInfo::new(&self.headers, &args, &self.url);
            let docs = Documents::query(&doc_info, &self.user).await?;
            let links = DocumentLinks::from(&docs);
            let mut linked = WebLinks::from(&links);
            let link_path = format!("{}/{}.csv", self.output, file);
            linked.to_csv(&link_path)?;
            info!("Links printed to {}", &link_path);
        } else {
            warn!("Folder name {} not found.", folder);
        }
        Ok(())
    }
}
