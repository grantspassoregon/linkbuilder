use crate::{document, error, import, utils};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebLink {
    field: String,
    web_link: std::path::PathBuf,
}

impl WebLink {
    pub fn new(field: &str, link: &std::path::PathBuf) -> Self {
        WebLink {
            field: field.to_owned(),
            web_link: link.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebLinks {
    records: Vec<WebLink>,
}

impl WebLinks {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilaLink {
    object_id: i64,
    instrument: String,
    global_id: String,
    web_link: std::path::PathBuf,
}

impl FilaLink {
    pub fn new(fila: &import::Fila, link: &std::path::PathBuf) -> Self {
        FilaLink {
            object_id: fila.object_id(),
            instrument: fila.instrument(),
            global_id: fila.global_id(),
            web_link: link.to_owned(),
        }
    }
    pub fn from_links(
        fila: &import::Fila,
        links: &document::DocumentLinks,
    ) -> Result<Self, error::LinkError> {
        let records = links
            .ref_links()
            .iter()
            .filter(|(k, _)| k == &fila.instrument_ref())
            .map(|(_, v)| FilaLink::new(fila, v))
            .take(1)
            .collect::<Vec<FilaLink>>();
        if !records.is_empty() {
            Ok(records[0].clone())
        } else {
            warn!("No link for {}.", fila.instrument_ref());
            Err(error::LinkError::MissingLink)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilaLinks {
    records: Vec<FilaLink>,
}

impl FilaLinks {
    pub fn from_links(
        filas: &import::Filas,
        links: &document::DocumentLinks,
    ) -> Result<Self, error::LinkError> {
        let mut records = Vec::new();
        for fila in filas.records_ref() {
            if let Ok(link) = FilaLink::from_links(fila, links) {
                records.push(link);
            }
        }
        Ok(FilaLinks { records })
    }

    pub fn to_csv<P: AsRef<std::path::Path>>(&mut self, title: P) -> Result<(), std::io::Error> {
        utils::to_csv(&mut self.records, title)?;
        Ok(())
    }
}
