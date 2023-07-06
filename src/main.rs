use linkbuilder::{authorize, document, error, file};
use tracing::{info, trace};

#[tokio::main]
async fn main() -> Result<(), error::LinkError> {
    if let Ok(()) = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
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
    let fila = std::path::PathBuf::from(std::env::var("FEE_IN_LIEU")?);
    trace!("Environmental variables loaded.");

    let names = file::FileNames::from_path("p:/fila".into())?;
    info!("Names read: {:?}", names.names().len());

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
    let url = std::env::var("FOLDER")?;
    let mut args = document::DocQuery::new();
    // args.inlinecount("allpages").filter("FolderId eq 1797");
    args.inlinecount("allpages");
    let doc_info = document::DocInfo::new(&doc_header, &args, &url);
    let folders = document::Folders::query(&doc_info, &auth_user).await?;

    let url = std::env::var("DOCUMENT")?;
    // if let Some(id) = folders.get_id("Fee in Lieu") {
    if let Some(id) = folders.get_id("Fee in Lieu") {
        info!("Folder id: {:?}", id);
        args.filter(&format!("FolderId eq {}", id));
        let doc_info = document::DocInfo::new(&doc_header, &args, &url);
        let docs = document::Documents::query(&doc_info, &auth_user).await?;
        info!("Total size: {}", docs.total_size());
        let links = document::DocumentLinks::from(&docs);
        info!("Links read: {:?}", links.ref_links().len());
        let diff = names.not_in(&links);
        info!("Names not in doc links: {:?}", diff.names().len());
        // if let Some(source) = docs.source() {
        //     info!("Example file: {:?}", source[0]);
        // }
        diff.upload(&url, &doc_info, &auth_user, id).await?;
        info!("upload attempted.")

    } else {
        info!("Folder not present.");
    };
    // let docs = document::Documents::query(&doc_info, &auth_user).await?;
    // if let Some(value) = docs.total_count() {
    //     println!("Total count: {}", value);
    // }
    // if let Some(value) = docs.source() {
    //     println!("Docs: {}", value.len());
    // }

    Ok(())
}
