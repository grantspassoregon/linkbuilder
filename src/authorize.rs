use crate::error;
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::json;
use tracing::{trace, warn};

/// Struct containing user attributes for logging into CivicEngage.
#[derive(Clone)]
pub struct User {
    api_key: String,
    partition: String,
    name: String,
    password: String,
    host: String,
}

impl User {
    /// Creates a UserBuilder struct, allowing incremental construction of a User type.
    pub fn new() -> UserBuilder {
        UserBuilder::new()
    }
}

/// Struct UserBuilder contains optional fields for incrementally adding parameters to a User type.
pub struct UserBuilder {
    api_key: Option<String>,
    partition: Option<String>,
    name: Option<String>,
    password: Option<String>,
    host: Option<String>,
}

impl UserBuilder {
    /// Called by [`User`] to create a UserBuilder struct for building a User type. Calls
    /// [`UserBuilder::default()`].
    pub fn new() -> Self {
        UserBuilder::default()
    }

    /// The `api_key` field contains the user API key for CivicEngage.  This function sets the
    /// value of the UserBuilder field to `value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::User;
    /// let user = User::new()
    ///     .api_key("Some key");
    /// ```
    pub fn api_key(&mut self, value: &str) -> &mut Self {
        self.api_key = Some(value.to_string());
        self
    }

    /// The `partition` field contains the partition number assigned to the user by CivicEngage.  This function sets the
    /// value of the UserBuilder field to `value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::User;
    /// let user = User::new()
    ///     .partition("1234");
    /// ```
    pub fn partition(&mut self, value: &str) -> &mut Self {
        self.partition = Some(value.to_string());
        self
    }

    /// The `name` field contains the username for CivicEngage.  This function sets the
    /// value of the UserBuilder field to `value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::User;
    /// let user = User::new()
    ///     .name("Your username");
    /// ```
    pub fn name(&mut self, value: &str) -> &mut Self {
        self.name = Some(value.to_string());
        self
    }

    /// The `password` field contains the user password for CivicEngage.  This function sets the
    /// value of the UserBuilder field to `value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::User;
    /// let user = User::new()
    ///     .password("Your password");
    /// ```
    pub fn password(&mut self, value: &str) -> &mut Self {
        self.password = Some(value.to_string());
        self
    }

    /// The `host` field contains the user domain for CivicEngage.  This function sets the
    /// value of the UserBuilder field to `value`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::User;
    /// let user = User::new()
    ///     .host("grantspassoregon.gov");
    /// ```
    pub fn host(&mut self, value: &str) -> &mut Self {
        self.host = Some(value.to_string());
        self
    }

    /// Converts UserBuilder into User.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::{LinkResult, User};
    /// # fn main() -> LinkResult<()> {
    /// let user = User::new()
    ///     .api_key("Some key")
    ///     .partition("1234")
    ///     .name("Your username")
    ///     .password("Your password")
    ///     .host("grantspassoregon.gov")
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
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

/// Headers for authorizing a user on CivicEngage.
pub struct AuthorizeHeaders {
    api_key: HeaderName,
    partition: HeaderName,
}

impl AuthorizeHeaders {
    /// Create authorization headers from string values for `api_key` and `partition`.  CivicEngage
    /// issues these values to approved users.  Contact your system administrator for access
    /// credentials.
    pub fn new(api_key: &'static str, partition: &'static str) -> Self {
        let api_key_header = HeaderName::from_static(api_key);
        let partition_header = HeaderName::from_static(partition);
        AuthorizeHeaders {
            api_key: api_key_header,
            partition: partition_header,
        }
    }

    /// The `api_key` field holds the API key assigned to the user by CivicEngage.  This function
    /// returns the set value of the field.
    pub fn api_key(self) -> HeaderName {
        self.api_key
    }

    /// The `partition` field holds the partition number assigned to the user by CivicEngage.  This function
    /// returns the set value of the field.
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

/// Struct holding user info and authorization headers for authorization calls to CivicEngage.
pub struct AuthorizeInfo {
    user: User,
    headers: AuthorizeHeaders,
}

impl AuthorizeInfo {
    /// Creates a new AuthorizeInfo struct from a reference to a `User` and an instance of
    /// AuthorizeHeaders.
    pub fn new(user: &User, headers: AuthorizeHeaders) -> Self {
        AuthorizeInfo {
            user: user.clone(),
            headers,
        }
    }

    /// Logs user into CivicEngage and returns authorization response containing session id.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::{AuthorizeHeaders, AuthorizeInfo, LinkError, LinkResult, User};
    /// # #[tokio::main]
    /// # async fn main() -> LinkResult<()> {
    /// let user = User::new()
    ///     .api_key("Some key")
    ///     .partition("1234")
    ///     .name("Your username")
    ///     .password("Your password")
    ///     .host("grantspassoregon.gov")
    ///     .build()?;
    /// let headers = AuthorizeHeaders::default();
    /// let auth_info = AuthorizeInfo::new(&user, headers);
    /// let url = "https://www.grantspassoregon.gov/api/Authentication/v1/Authenticate";
    /// let response = auth_info.authorize(url).await.expect_err("Invalid credentials.");
    /// # Ok(())
    /// # }
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

/// Struct for holding authorization responses from CivicEngage. Returned by
/// [`AuthorizeInfo::authorize`].
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
    /// The `additional_info` field holds the additionalInfo content from the response.  This
    /// function returns the value of the field.
    pub fn info(&self) -> String {
        self.additional_info.clone()
    }
    /// The `success` field is a bool indicating if the response was a success.  This
    /// function returns the value of the field.
    pub fn success(&self) -> bool {
        self.success
    }
    /// The `api_key` field holds the session id value of the response.  This
    /// function returns the value of the field.
    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }
    /// The `id` field holds the user id.  This function returns the value of the field.
    pub fn id(&self) -> i32 {
        self.user_id
    }
    /// The `message` field holds the message associated with the response.  This function returns the value of the field.
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

/// Struct holding credentials for authorized users on CivicEngage.
pub struct AuthorizedUser {
    api_key: String,
    partition: String,
    user_api_key: String,
}

impl AuthorizedUser {
    /// Create an `AuthorizedUser` by passing the [`User`] and [`AuthResponse`] into the function.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use linkbuilder::prelude::{AuthorizeHeaders, AuthorizeInfo, AuthorizedUser, LinkError, LinkResult, User};
    /// # #[tokio::main]
    /// # async fn main() -> LinkResult<()> {
    /// dotenv::dotenv().ok();
    /// let api_key = std::env::var("API_KEY")?;
    /// let partition = std::env::var("PARTITION")?;
    /// let name = std::env::var("USERNAME")?;
    /// let password = std::env::var("PASSWORD")?;
    /// let host = std::env::var("HOST")?;
    ///
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
    /// # Ok(())
    /// # }
    pub fn new(user: &User, auth: &AuthResponse) -> Self {
        AuthorizedUser {
            api_key: user.api_key.clone(),
            partition: user.partition.clone(),
            user_api_key: auth.api_key.clone(),
        }
    }

    /// The `api_key` field holds the user API key issued by CivicEngage.  This functions returns
    /// the value of the field.
    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }

    /// The `partition` field holds the user partition number issued by CivicEngage.  This functions returns
    /// the value of the field.
    pub fn partition(&self) -> String {
        self.partition.clone()
    }

    /// The `user_api_key` field holds the user session id from the [`AuthResponse`].  This functions returns
    /// the value of the field.
    pub fn user_api_key(&self) -> String {
        self.user_api_key.clone()
    }
}
