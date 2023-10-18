use clap::Parser;
use linkbuilder::{authorize, document, error, export, file, report};
use tracing::{info, trace, warn};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'c', long, help = "Command to execute.")]
    command: String,
    #[arg(short = 'p', long, help = "Parameter for command.")]
    param: String,
    #[arg(short = 's', long, help = "Source path.")]
    source: Option<String>,
    #[arg(short = 'o', long, help = "Output path.")]
    output: Option<String>,
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

    trace!("Preparing document center headers.");
    let doc_header = document::DocumentHeaders::default();
    trace!("Setting query parameters for document center request.");
    let mut args = document::DocQuery::new();
    trace!("Returns all matches on server.");
    args.inlinecount("allpages");

    let cli = Cli::parse();
    match cli.command.as_str() {
        "get_links" => {
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;
            trace!("Search for docs in specified folder.");
            let url = std::env::var("DOCUMENT")?;
            if let Some(id) = folders.get_id("Fee in Lieu") {
                trace!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                let links = document::DocumentLinks::from(&docs);
                let mut linked = export::WebLinks::from(&links);
                if let Some(path) = cli.output.clone() {
                    let link_path = format!("{}/fila_links.csv", path.clone());
                    linked.to_csv(&link_path)?;
                    info!("Links printed to {}", &link_path);
                } else {
                    warn!("No output path provided for results.");
                }
            } else {
                warn!("Fee in Lieu folder not found.");
            }
            if let Some(id) = folders.get_id("Unrecorded Parcels") {
                trace!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                let links = document::DocumentLinks::from(&docs);
                let mut linked = export::WebLinks::from(&links);
                if let Some(path) = cli.output.clone() {
                    let link_path = format!("{}/unrecorded_parcels_links.csv", path.clone());
                    linked.to_csv(&link_path)?;
                    info!("Links printed to {}", &link_path);
                } else {
                    warn!("No output path provided for results.");
                }
            } else {
                warn!("Unrecorded parcels folder not found.");
            }
            if let Some(id) = folders.get_id("Service and Annexation") {
                trace!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                let links = document::DocumentLinks::from(&docs);
                let mut linked = export::WebLinks::from(&links);
                if let Some(path) = cli.output.clone() {
                    let link_path = format!("{}/service_annexation_links.csv", path.clone());
                    linked.to_csv(&link_path)?;
                    info!("Links printed to {}", &link_path);
                } else {
                    warn!("No output path provided for results.");
                }
            } else {
                warn!("Service and Annexation folder not found.");
            }
            if let Some(id) = folders.get_id("Deferred Development Agreements") {
                trace!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                let links = document::DocumentLinks::from(&docs);
                let mut linked = export::WebLinks::from(&links);
                if let Some(path) = cli.output.clone() {
                    let link_path = format!("{}/deferred_development_links.csv", path.clone());
                    linked.to_csv(&link_path)?;
                    info!("Links printed to {}", &link_path);
                } else {
                    warn!("No output path provided for results.");
                }
            } else {
                warn!("Deferred Development Agreements folder not found.");
            }
            if let Some(id) = folders.get_id("Advance Finance Districts") {
                trace!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                let links = document::DocumentLinks::from(&docs);
                let mut linked = export::WebLinks::from(&links);
                if let Some(path) = cli.output {
                    let link_path = format!("{}/advance_finance_links.csv", path.clone());
                    linked.to_csv(&link_path)?;
                    info!("Links printed to {}", &link_path);
                } else {
                    warn!("No output path provided for results.");
                }
            } else {
                warn!("Advance Finance Districts folder not found.");
            }
        }
        "sync_folder" => {
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;

            trace!("Reading files in source directory.");
            if let Some(path) = cli.source {
                let names = file::FileNames::from_path(path)?;
                trace!("Names read: {:?}", names.names().len());

                trace!("Search for docs in specified folder.");
                if let Some(id) = folders.get_id(&cli.param) {
                    trace!("Folder id: {:?}", id);
                    trace!("Specify folder for search.");
                    args.filter(&format!("FolderId eq {}", id));
                    let url = std::env::var("DOCUMENT")?;
                    let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                    let docs = document::Documents::query(&doc_info, &auth_user).await?;

                    if let Some(count) = docs.total_count() {
                        info!("Total count of documents in folder: {}", count);
                    }
                    info!("Total size of documents in folder: {}", docs.total_size());
                    let links = document::DocumentLinks::from(&docs);
                    info!("Links read: {:?}", links.ref_links().len());
                    info!("Names found: {:?}", links.ref_links().keys());
                    trace!("Comparing names of docs in web folder to names in local folder.");
                    let diff = names.not_in(&links);
                    info!("Local names not in web folder: {:?}", diff.names().len());
                    let res = diff.upload(&url, &doc_info, &auth_user, id).await?;
                    info!("Files added to web folder: {:?}", res.len());
                }
            } else {
                info!("Source path not specified.")
            }
        }
        "report" => {
            info!("Preparing report.");
            let mut records = Vec::new();
            let url = std::env::var("DOCUMENT")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            let total = document::Documents::query(&doc_info, &auth_user).await?;
            let folder_list = vec![
                "GIS",
                "Address Notifications",
                "Images",
                "Advance Finance Districts",
                "Deferred Development Agreements",
                "Fee in Lieu",
                "Service and Annexation",
                "Unrecorded Parcels",
            ];
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;
            for folder in folder_list {
                if let Some(id) = folders.get_id(folder) {
                    args.filter(&format!("FolderId eq {}", id));
                    let url = std::env::var("DOCUMENT")?;
                    let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                    let docs = document::Documents::query(&doc_info, &auth_user).await?;
                    records.push(report::FolderSize::new(folder, docs.total_size()));
                } else {
                    info!("Could not find folder: {}.", folder);
                }
            }
            let subtotal = report::FolderSizes::from(records.clone()).size();
            records.push(report::FolderSize::new("Subtotal", subtotal));
            records.push(report::FolderSize::new("Total", total.total_size()));
            let sizes = report::FolderSizes::from(records);
            if let Ok(mut report) = report::ReportItems::try_from(sizes) {
                if let Some(path) = cli.output {
                    report.to_csv(path.clone())?;
                    info!("Report output to path: {}", path)
                }
            }
        }
        "folder_count" => {
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;

            trace!("Search for docs in specified folder.");
            if let Some(id) = folders.get_id(&cli.param) {
                info!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                trace!("Querying documents in folder.");
                let url = std::env::var("DOCUMENT")?;
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                if let Some(count) = docs.total_count() {
                    info!("Total count of documents in folder: {}", count);
                }
                info!("Total size of documents in folder: {}", docs.total_size());
                if let Some(items) = docs.source_ref() {
                    for item in items {
                        info!("Name: {}", item.name());
                        if let Some(size) = item.file_size() {
                            info!("Size: {}", size);
                        }
                        if let Some(path) = item.url_ref() {
                            info!("Url: {}", path);
                        }
                    }
                }
            } else {
                info!("Folder not present.");
            };
        }
        "delete_folder_content" => {
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;

            trace!("Search for docs in specified folder.");
            if let Some(id) = folders.get_id(&cli.param) {
                trace!("Folder id: {:?}", id);
                trace!("Specify folder for search.");
                args.filter(&format!("FolderId eq {}", id));
                trace!("Querying documents in folder.");
                let url = std::env::var("DOCUMENT")?;
                let doc_info = document::DocInfo::new(&doc_header, &args, &url);
                let docs = document::Documents::query(&doc_info, &auth_user).await?;
                let res = docs.update(&url, &doc_info, &auth_user, "draft").await?;
                trace!("Response: {:?}", res);
                let res = docs.delete(&url, &doc_info, &auth_user).await?;
                trace!("Response: {:?}", res);
            } else {
                info!("Folder not present.");
            };
        }
        "inspect_folder" => {
            let url = std::env::var("FOLDER")?;
            let doc_info = document::DocInfo::new(&doc_header, &args, &url);
            trace!("Set up query data for folders.");
            let folders = document::Folders::query(&doc_info, &auth_user).await?;

            trace!("Search for docs in specified folder.");
            if let Some(id) = folders.get_id(&cli.param) {
                if let Some(items) = folders.source() {
                    let folder = items
                        .iter()
                        .filter(|i| i.id_ref() == &Some(id))
                        .collect::<Vec<&document::Folder>>();
                    if !folder.is_empty() {
                        info!("{:#?}", folder);
                    }
                }
            }
        }

        _ => {}
    }

    Ok(())
}
