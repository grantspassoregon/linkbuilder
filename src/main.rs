use error_chain::error_chain;
use linkbuilder::authorize;
use reqwest::header::{HeaderName, ACCEPT, CONTENT_TYPE};
use serde_json::json;
use tracing::info;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(()) = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .try_init()
    {};
    info!("Subscriber initialized.");
    info!("Loading environmental variables.");
    dotenv::dotenv().ok();
    info!("Environmental variables loaded.");

    let api_key = std::env::var("API_KEY").expect("API key must be set.");
    let partition = std::env::var("PARTITION").expect("Partition must be set.");
    let name = std::env::var("USERNAME").expect("Username must be set.");
    let password = std::env::var("PASSWORD").expect("Password must be set.");
    let host = std::env::var("HOST").expect("Host must be set.");
    let user = authorize::User::new()
        .api_key(&api_key)
        .partition(&partition)
        .name(&name)
        .password(&password)
        .host(&host)
        .build()
        .unwrap();

    let headers = authorize::AuthorizeHeaders::default();

    let url = std::env::var("AUTHENTICATE").expect("Authenticate REST endpoint must be set.");

    let auth_info = authorize::AuthorizeInfo::new(user, headers);

    let url = "https://www.grantspassoregon.gov/api/Authentication/v1/Authenticate";
    let client = reqwest::Client::new();
    info!("Client created.");
    let api_key_header = HeaderName::from_static("apikey");
    let api_key = std::env::var("API_KEY").expect("API key must be set.");
    let partition_header = HeaderName::from_static("partition");
    let partition = std::env::var("PARTITION").expect("Partition must be set.");
    let username = std::env::var("USERNAME").expect("Username must be set.");
    let username = format!("{}@grantspassoregon.gov", username);
    let password = std::env::var("PASSWORD").expect("Password must be set.");
    // let mut auth_map = std::collections::HashMap::new();
    // auth_map.insert(username, password);
    let auth = json!({
        "Username": username,
        "Password": password
    });

    info!("Sending request with body {:?}.", auth.to_string());
    let res = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .header(api_key_header, api_key)
        .header(partition_header, partition)
        .body(auth.to_string())
        .send()
        .await?;
    println!("Status: {}", res.status());
    match res.status() {
        reqwest::StatusCode::NOT_FOUND => {
            println!("Status: {}", res.status());
        }
        reqwest::StatusCode::OK => {
            println!("Status: {}", res.status());
            println!("Headers:\n{:#?}", res.headers());
        }
        _ => {
            println!("Status: {}", res.status());
        }
    }

    // let res = reqwest::get("https://www.grantspassoregon.gov/DocumentCenter/View/26449/803_SE_8TH_ST_FINAL").await?;
    // println!("Status: {}", res.status());
    // println!("Headers:\n{:#?}", res.headers());
    //
    // let body = res.text().await?;
    // println!("Body:\n{}", body);
    Ok(())
}
