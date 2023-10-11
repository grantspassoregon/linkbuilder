use clap::Parser;
use linkbuilder::{authorize, document, error, file};
use tracing::{info, trace};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'c', long, help = "Command to execute.")]
    command: String,
    #[arg(short = 'p', long, help = "Parameter for command.")]
    param: String,
}

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

    // let notifs = std::path::PathBuf::from(std::env::var("NOTIFICATION")?);
    // let fila = std::path::PathBuf::from(std::env::var("FEE_IN_LIEU")?);
    trace!("Environmental variables loaded.");

    info!("Authorizing user...");
    trace!("Creating user from environmental variables.");
    let user = authorize::User::new()
        .api_key(&api_key)
        .partition(&partition)
        .name(&name)
        .password(&password)
        .host(&host)
        .build()?;

    trace!("Preparing authorization headers.");
    let headers = authorize::AuthorizeHeaders::default();
    trace!("Authorizing user.");
    let auth_info = authorize::AuthorizeInfo::new(&user, headers);
    let url = std::env::var("AUTHENTICATE")?;
    let auth_res = auth_info.authorize(&url).await?;
    info!("Authorization successful for user {}.", &auth_res.id());
    trace!("Recording session id of user.");
    let auth_user = authorize::AuthorizedUser::new(&user, &auth_res);

    let cli = Cli::parse();
    match cli.command.as_str() {
        "delete_folder_content" => {
            trace!("Preparing document center headers.");
            let doc_header = document::DocumentHeaders::default();
            trace!("Setting query parameters for document center request.");
            let mut args = document::DocQuery::new();
            trace!("Returns all matches on server.");
            args.inlinecount("allpages");
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;

            trace!("Reading files in source directory.");
            let names = file::FileNames::from_path("p:/fila".into())?;
            trace!("Names read: {:?}", names.names().len());

            trace!("Search for docs in specified folder.");
            if let Some(id) = folders.get_id(&cli.param) {
                info!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                trace!("Querying documents in folder.");
                let url = std::env::var("DOCUMENT")?;
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                info!("Total size of documents in folder: {}", docs.total_size());
                let links = document::DocumentLinks::from(&docs);
                info!("Links read: {:?}", links.ref_links().len());
                info!("Names found: {:?}", links.ref_links().keys());
                trace!("Comparing names of docs in web folder to names in local folder.");
                let diff = names.not_in(&links);
                info!("Local names not in web folder: {:?}", diff.names().len());
                // diff.upload(&url, &doc_info, &auth_user, id).await?;
            } else {
                info!("Folder not present.");
            };
        }
        "sync_folder" => {}
        _ => {}
    }

    // trace!("Reading files in source directory.");
    // let names = file::FileNames::from_path("p:/fila".into())?;
    // trace!("Names read: {:?}", names.names().len());
    //
    //
    // trace!("Preparing document center headers.");
    // let doc_header = document::DocumentHeaders::default();
    // trace!("Setting query parameters for document center request.");
    // let mut args = document::DocQuery::new();
    // trace!("Returns all matches on server.");
    // args.inlinecount("allpages");
    // let url = std::env::var("FOLDER")?;
    // let doc_info = document::DocInfo::new(&doc_header, &args, &url);
    // trace!("Set up query data for folders.");
    // let folders = document::Folders::query(&doc_info, &auth_user).await?;

    // trace!("Count total documents on website and total size.");
    // let url = std::env::var("DOCUMENT")?;
    // let doc_info = document::DocInfo::new(&doc_header, &args, &url);
    // let docs = document::Documents::query(&doc_info, &auth_user).await?;
    // if let Some(value) = docs.total_count() {
    //     info!("Total documents on website: {}", value);
    // }
    // info!("Total size of documents on site: {}", docs.total_size());

    // trace!("Search for docs in specified folder.");
    // if let Some(id) = folders.get_id("Fee in Lieu") {
    //     info!("Folder id: {:?}", id);
    //     trace!("Specify folder for search.");
    //     args.filter(&format!("FolderId eq {}", id));
    //     trace!("Querying documents in folder.");
    //     let doc_info = document::DocInfo::new(&doc_header, &args, &url);
    //     let docs = document::Documents::query(&doc_info, &auth_user).await?;
    //     info!("Total size of documents in folder: {}", docs.total_size());
    //     let links = document::DocumentLinks::from(&docs);
    //     info!("Links read: {:?}", links.ref_links().len());
    //     info!("Names found: {:?}", links.ref_links().keys());
    //     trace!("Comparing names of docs in web folder to names in local folder.");
    //     let diff = names.not_in(&links);
    //     info!("Local names not in web folder: {:?}", diff.names().len());
    //     trace!("Uploading missing files to web folder.");
    //     diff.upload(&url, &doc_info, &auth_user, id).await?;
    //     info!("upload attempted.")
    // } else {
    //     info!("Folder not present.");
    // };

    Ok(())
}
