use crate::error;
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use tracing::{trace, warn};

#[derive(Clone)]
pub struct User {
    api_key: String,
    partition: String,
    name: String,
    password: String,
    host: String,
}

impl User {
    pub fn new() -> UserBuilder {
        UserBuilder::new()
    }
}

pub struct UserBuilder {
    api_key: Option<String>,
    partition: Option<String>,
    name: Option<String>,
    password: Option<String>,
    host: Option<String>,
}

impl UserBuilder {
    pub fn new() -> Self {
        UserBuilder::default()
    }

    pub fn api_key(&mut self, value: &str) -> &mut Self {
        self.api_key = Some(value.to_string());
        self
    }

    pub fn partition(&mut self, value: &str) -> &mut Self {
        self.partition = Some(value.to_string());
        self
    }

    pub fn name(&mut self, value: &str) -> &mut Self {
        self.name = Some(value.to_string());
        self
    }

    pub fn password(&mut self, value: &str) -> &mut Self {
        self.password = Some(value.to_string());
        self
    }

    pub fn host(&mut self, value: &str) -> &mut Self {
        self.host = Some(value.to_string());
        self
    }

    pub fn build(&mut self) -> Result<User, error::LinkError> {
        let mut fields = Vec::new();
        if self.api_key == None {
            fields.push("api_key".to_string());
        }
        if self.partition == None {
            fields.push("partition".to_string());
        }
        if self.name == None {
            fields.push("name".to_string());
        }
        if self.password == None {
            fields.push("password".to_string());
        }
        if self.host == None {
            fields.push("host".to_string());
        }
        if let Some(api_key) = &self.api_key {
            if let Some(partition) = &self.partition {
                if let Some(name) = &self.name {
                    if let Some(password) = &self.password {
                        if let Some(host) = &self.host {
                            Ok(User {
                                api_key: api_key.clone(),
                                partition: partition.clone(),
                                name: name.clone(),
                                password: password.clone(),
                                host: host.clone(),
                            })
                        } else {
                            Err(error::LinkError::UserBuildError { value: fields })
                        }
                    } else {
                        Err(error::LinkError::UserBuildError { value: fields })
                    }
                } else {
                    Err(error::LinkError::UserBuildError { value: fields })
                }
            } else {
                Err(error::LinkError::UserBuildError { value: fields })
            }
        } else {
            Err(error::LinkError::UserBuildError { value: fields })
        }
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        UserBuilder {
            api_key: None,
            partition: None,
            name: None,
            password: None,
            host: None,
        }
    }
}

pub struct AuthorizeHeaders {
    api_key: HeaderName,
    partition: HeaderName,
}

impl AuthorizeHeaders {
    pub fn new(api_key: &'static str, partition: &'static str) -> Self {
        let api_key_header = HeaderName::from_static(api_key);
        let partition_header = HeaderName::from_static(partition);
        AuthorizeHeaders {
            api_key: api_key_header,
            partition: partition_header,
        }
    }

    pub fn api_key(self) -> HeaderName {
        self.api_key
    }

    pub fn partition(self) -> HeaderName {
        self.partition
    }
}

impl Default for AuthorizeHeaders {
    fn default() -> Self {
        let api_key = HeaderName::from_static("apikey");
        let partition = HeaderName::from_static("partition");
        AuthorizeHeaders { api_key, partition }
    }
}

pub struct AuthorizeInfo {
    user: User,
    headers: AuthorizeHeaders,
}

impl AuthorizeInfo {
    pub fn new(user: &User, headers: AuthorizeHeaders) -> Self {
        AuthorizeInfo {
            user: user.clone(),
            headers,
        }
    }

    pub async fn authorize(&self, url: &str) -> Result<AuthResponse, error::LinkError> {
        let client = reqwest::Client::new();
        trace!("Authorization client created.");
        let username = format!("{}@{}", self.user.name, self.user.host);
        let body = json!({
            "Username": username,
            "Password": self.user.password
        });
        let res = client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .header(self.headers.api_key.clone(), self.user.api_key.clone())
            .header(self.headers.partition.clone(), self.user.partition.clone())
            .body(body.to_string())
            .send()
            .await?;
        match &res.status() {
            &reqwest::StatusCode::OK => Ok(res.json::<AuthResponse>().await?),
            _ => {
                warn!("Status: {}", res.status());
                Err(error::LinkError::AuthError)
            }
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct AuthResponse {
    additional_info: String,
    success: bool,
    #[serde(rename(deserialize = "APIKey"))]
    api_key: String,
    user_id: i32,
    message: String,
}

impl AuthResponse {
    pub fn info(&self) -> String {
        self.additional_info.clone()
    }
    pub fn success(&self) -> bool {
        self.success
    }
    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }
    pub fn id(&self) -> i32 {
        self.user_id
    }
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

pub struct AuthorizedUser {
    api_key: String,
    partition: String,
    user_api_key: String,
}

impl AuthorizedUser {
    pub fn new(user: &User, auth: &AuthResponse) -> Self {
        AuthorizedUser {
            api_key: user.api_key.clone(),
            partition: user.partition.clone(),
            user_api_key: auth.api_key.clone(),
        }
    }

    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn partition(&self) -> String {
        self.partition.clone()
    }

    pub fn user_api_key(&self) -> String {
        self.user_api_key.clone()
    }
}
