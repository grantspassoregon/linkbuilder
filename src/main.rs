use clap::Parser;
use linkbuilder::prelude::*;
use tracing::{info, trace};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'c', long, help = CMD_HELP)]
    command: String,
    #[arg(short = 'p', long, help = "Parameter for command.")]
    param: Option<String>,
    #[arg(short = 's', long, help = "Source path.")]
    source: Option<String>,
    #[arg(short = 'o', long, help = "Output path.")]
    output: Option<String>,
}

const CMD_HELP: &str = "
Command to execute, including:
* get_links -p <PATH> -> Read links from website and output links files to path.
* sync_folder -s <LOCAL_FILE_PATH> -p <WEB_FOLDER_NAME> -> Copies local files to web folder if not already present.
* report -o <PATH> -> Outputs a report of storage use for GIS on CivicEngage.
* folder_count -p <WEB_FOLDER_NAME> -> Prints stats about a folder contents.
* delete_folder_content -p <WEB_FOLDER_NAME> -> Deletes all contents from web folder.
* inspect_folder -p <WEB_FOLDER_NAME> -> Prints stats about a folder.
";

#[tokio::main]
async fn main() -> LinkResult<()> {
    dotenv::dotenv().ok();
    if let Ok(()) = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init()
    {};
    trace!("Subscriber initialized.");
    let folder_url = std::env::var("FOLDER")?;
    let doc_url = std::env::var("DOCUMENT")?;

    let auth_user = load_user().await?;

    trace!("Preparing document center headers.");
    let doc_header = DocumentHeaders::default();
    trace!("Setting query parameters for document center request.");
    let mut args = DocQuery::new();
    trace!("Returns all matches on server.");
    args.inlinecount("allpages");

    let cli = Cli::parse();
    match cli.command.as_str() {
        "get_links" => {
            let doc_info = DocInfo::new(&doc_header, &args, &folder_url);
            trace!("Set up query data for folders.");
            let folders = Folders::query(&doc_info, &auth_user).await?;
            trace!("Search for docs in specified folder.");

            let link_updater = LinkUpdater::new()
                .folders(&folders)
                .headers(&doc_header)
                .args(&args)
                .url(&doc_url)
                .user(&auth_user)
                .output(&cli.output)?
                .build()?;
            link_updater
                .get_links("Advance Finance Districts", "advance_finance_links")
                .await?;
            link_updater
                .get_links(
                    "Deferred Development Agreements",
                    "deferred_development_links",
                )
                .await?;
            link_updater.get_links("Fee in Lieu", "fila_links").await?;
            link_updater.get_links("Plats", "plat_links").await?;
            link_updater
                .get_links("Service and Annexation", "service_annexation_links")
                .await?;
            link_updater
                .get_links("Unrecorded Parcels", "unrecorded_parcels_links")
                .await?;
            info!("Links successfully updated.");
        }
        "sync_folder" => {
            let doc_info = DocInfo::new(&doc_header, &args, &folder_url);
            trace!("Set up query data for folders.");
            let folders = Folders::query(&doc_info, &auth_user).await?;

            trace!("Reading files in source directory.");
            if let Some(path) = cli.source {
                let names = FileNames::from_path(path)?;
                trace!("Names read: {:?}", names.names().len());

                trace!("Search for docs in specified folder.");
                if let Some(folder) = &cli.param {
                    if let Some(id) = folders.get_id(folder) {
                        trace!("Folder id: {:?}", id);
                        trace!("Specify folder for search.");
                        args.filter(&format!("FolderId eq {}", id));
                        let doc_info = DocInfo::new(&doc_header, &args, &doc_url);
                        let docs = Documents::query(&doc_info, &auth_user).await?;

                        if let Some(count) = docs.total_count() {
                            info!("Total count of documents in folder: {}", count);
                        }
                        info!("Total size of documents in folder: {}", docs.total_size());
                        let links = DocumentLinks::from(&docs);
                        info!("Links read: {:?}", links.ref_links().len());
                        info!("Names found: {:?}", links.ref_links().keys());
                        trace!("Comparing names of docs in web folder to names in local folder.");
                        let diff = names.not_in(&links);
                        info!("Local names not in web folder: {:?}", diff.names().len());
                        let res = diff.upload(&doc_info, &auth_user, id).await?;
                        info!("Files added to web folder: {:?}", res.len());
                    }
                }
            } else {
                info!("Source path not specified.")
            }
        }
        "report" => {
            info!("Preparing report.");
            let mut records = Vec::new();
            let doc_info = DocInfo::new(&doc_header, &args, &doc_url);
            let total = Documents::query(&doc_info, &auth_user).await?;
            let folder_list = vec![
                "GIS",
                "Address Notifications",
                "Advance Finance Districts",
                "Deferred Development Agreements",
                "Fee in Lieu",
                "Images",
                "Plats",
                "Service and Annexation",
                "Unrecorded Parcels",
            ];
            let doc_info = DocInfo::new(&doc_header, &args, &folder_url);
            trace!("Set up query data for folders.");
            let folders = Folders::query(&doc_info, &auth_user).await?;
            for folder in folder_list {
                if let Some(id) = folders.get_id(folder) {
                    args.filter(&format!("FolderId eq {}", id));
                    let doc_info = DocInfo::new(&doc_header, &args, &doc_url);
                    let docs = Documents::query(&doc_info, &auth_user).await?;
                    records.push(FolderSize::new(folder, docs.total_size()));
                } else {
                    info!("Could not find folder: {}.", folder);
                }
            }
            let subtotal = FolderSizes::from(records.clone()).size();
            records.push(FolderSize::new("Subtotal", subtotal));
            records.push(FolderSize::new("Total", total.total_size()));
            let sizes = FolderSizes::from(records);
            if let Ok(mut report) = ReportItems::try_from(sizes) {
                if let Some(path) = cli.output {
                    report.to_csv(path.clone())?;
                    info!("Report output to path: {}", path)
                }
            }
        }
        "folder_count" => {
            let doc_info = DocInfo::new(&doc_header, &args, &folder_url);
            trace!("Set up query data for folders.");
            let folders = Folders::query(&doc_info, &auth_user).await?;

            trace!("Search for docs in specified folder.");
            if let Some(folder) = &cli.param {
                if let Some(id) = folders.get_id(folder) {
                    info!("Folder id: {:?}", id);
                    trace!("Specify folder for search.");
                    args.filter(&format!("FolderId eq {}", id));
                    trace!("Querying documents in folder.");
                    let doc_info = DocInfo::new(&doc_header, &args, &doc_url);
                    let docs = Documents::query(&doc_info, &auth_user).await?;
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
                }
            };
        }
        "delete_folder_content" => {
            let doc_info = DocInfo::new(&doc_header, &args, &folder_url);
            trace!("Set up query data for folders.");
            let folders = Folders::query(&doc_info, &auth_user).await?;

            trace!("Search for docs in specified folder.");
            if let Some(folder) = &cli.param {
                if let Some(id) = folders.get_id(folder) {
                    trace!("Folder id: {:?}", id);
                    trace!("Specify folder for search.");
                    args.filter(&format!("FolderId eq {}", id));
                    trace!("Querying documents in folder.");
                    let doc_info = DocInfo::new(&doc_header, &args, &doc_url);
                    let docs = Documents::query(&doc_info, &auth_user).await?;
                    let res = docs.update(&doc_info, &auth_user, "draft").await?;
                    trace!("Response: {:?}", res);
                    let res = docs.delete(&doc_info, &auth_user).await?;
                    trace!("Response: {:?}", res);
                } else {
                    info!("Folder not present.");
                }
            };
        }
        "inspect_folder" => {
            let doc_info = DocInfo::new(&doc_header, &args, &folder_url);
            trace!("Set up query data for folders.");
            let folders = Folders::query(&doc_info, &auth_user).await?;

            trace!("Search for docs in specified folder.");
            if let Some(folder) = &cli.param {
                if let Some(id) = folders.get_id(folder) {
                    if let Some(items) = folders.source() {
                        let folder = items
                            .iter()
                            .filter(|i| i.id_ref() == &Some(id))
                            .collect::<Vec<&Folder>>();
                        if !folder.is_empty() {
                            info!("{:#?}", folder);
                        }
                    }
                }
            }
        }

        _ => {}
    }

    Ok(())
}
