#![warn(missing_docs)]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://www.grantspassoregon.gov/DocumentCenter/View/31368/GPLogo_450W-PNG"
)]
#![doc(html_playground_url = "https://play.rust-lang.org/")]
pub mod authorize;
pub mod document;
pub mod error;
/// Data types for exporting results to csv.
pub mod export;
/// Data types for reading file names from local folders.
pub mod file;
/// Reporting structure for storage on the CivicEngage Document Center.
pub mod report;
/// Generic functions accessed by internal modules.
pub mod utils;

/// Select set of common library features.
pub mod prelude {
    pub use crate::authorize::{AuthorizeHeaders, AuthorizeInfo, AuthorizedUser, User};
    pub use crate::document::{
        DocInfo, DocQuery, DocumentHeaders, DocumentLinks, Documents, Folder, Folders, LinkUpdater,
    };
    pub use crate::error::{LinkError, LinkResult};
    pub use crate::export::WebLinks;
    pub use crate::file::FileNames;
    pub use crate::report::{FolderSize, FolderSizes, ReportItems};
    pub use crate::utils::load_user;
}
