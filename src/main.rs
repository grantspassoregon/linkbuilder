use linkbuilder::{authorize, document, error, file};
use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<(), error::LinkError> {
    if let Ok(()) = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .try_init()
    {};
    trace!("Subscriber initialized.");

    trace!("Loading environmental variables.");
    dotenv::dotenv().ok();
    let api_key = std::env::var("API_KEY")?;
    let partition = std::env::var("PARTITION")?;
    let name = std::env::var("USERNAME")?;
    let password = std::env::var("PASSWORD")?;
    let host = std::env::var("HOST")?;

    let notifs = std::path::PathBuf::from(std::env::var("NOTIFICATION")?);
    trace!("Environmental variables loaded.");

    let names = file::FileNames::from_path(notifs)?;
    println!("Names are {:?}", names);

    let user = authorize::User::new()
        .api_key(&api_key)
        .partition(&partition)
        .name(&name)
        .password(&password)
        .host(&host)
        .build()?;

    let headers = authorize::AuthorizeHeaders::default();
    let url = std::env::var("AUTHENTICATE")?;
    let auth_info = authorize::AuthorizeInfo::new(&user, headers);
    let auth_res = auth_info.authorize(&url).await?;
    info!("Authorization successful for user {}.", &auth_res.id());
    let auth_user = authorize::AuthorizedUser::new(&user, &auth_res);

    let doc_header = document::DocumentHeaders::default();
    // let url = std::env::var("DOCUMENT")?;
    let url = std::env::var("FOLDER")?;
    let mut args = document::DocQuery::new();
    // args.inlinecount("allpages").filter("FolderId eq 1797");
    args.inlinecount("allpages");
    let doc_info = document::DocInfo::new(&doc_header, &args, &url);
    let folders = document::Folders::query(&doc_info, &auth_user).await?;
    if let Some(value) = folders.total_count() {
        println!("Total count: {}", value);
    }
    // let docs = document::Documents::query(&doc_info, &auth_user).await?;
    // if let Some(value) = docs.total_count() {
    //     println!("Total count: {}", value);
    // }
    // if let Some(value) = docs.source() {
    //     println!("Docs: {}", value.len());
    // }

    Ok(())
}
