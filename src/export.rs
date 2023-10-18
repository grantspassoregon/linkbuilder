use crate::{document, utils};
use serde::{Deserialize, Serialize};

/// Struct for holding web links to documents using a key `field`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebLink {
    field: String,
    web_link: std::path::PathBuf,
}

impl WebLink {
    /// Creates a `WebLink` from an identifier `field` and path `link`.  Called by [`WebLinks::from()`]
    pub fn new(field: &str, link: &std::path::PathBuf) -> Self {
        WebLink {
            field: field.to_owned(),
            web_link: link.to_owned(),
        }
    }
}

/// Holds a vector of [`WebLink`] objects.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebLinks {
    records: Vec<WebLink>,
}

impl WebLinks {
    /// Outputs values to csv file at path `title`.
    pub fn to_csv<P: AsRef<std::path::Path>>(&mut self, title: P) -> Result<(), std::io::Error> {
        utils::to_csv(&mut self.records, title)?;
        Ok(())
    }
}

impl From<&document::DocumentLinks> for WebLinks {
    fn from(doc: &document::DocumentLinks) -> Self {
        let records = doc
            .ref_links()
            .iter()
            .map(|(k, v)| WebLink::new(k, v))
            .collect::<Vec<WebLink>>();
        WebLinks { records }
    }
}
