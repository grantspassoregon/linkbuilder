use crate::{error, utils};
use serde::{Deserialize, Serialize};
use std::ops::Div;

/// Represents total size of dcouments in a Document Center folder.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FolderSize {
    folder: String,
    size: f64,
}

impl FolderSize {
    /// The field `size` represents the total storage in KB.
    pub fn size(&self) -> f64 {
        self.size
    }
}

impl FolderSize {
    /// Creates a new [`FolderSize`] based on the folder name (`folder`) and storage `size`.
    pub fn new(folder: &str, size: f64) -> Self {
        FolderSize {
            folder: folder.to_owned(),
            size,
        }
    }
}

/// Holds a vector of [`FolderSize`] objects.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FolderSizes {
    records: Vec<FolderSize>,
}

impl FolderSizes {
    /// Returns a clone of the underlying vector of [`FolderSize`] objects.
    pub fn records(&self) -> Vec<FolderSize> {
        self.records.clone()
    }

    /// Returns a reference to the underlying vector of [`FolderSize`] objects.
    pub fn records_ref(&self) -> &Vec<FolderSize> {
        &self.records
    }

    /// Returns the total size of all [`FolderSize`] objects.
    pub fn size(&self) -> f64 {
        self.records_ref().iter().fold(0.0, |acc, x| acc + x.size())
    }
}

impl From<Vec<FolderSize>> for FolderSizes {
    fn from(records: Vec<FolderSize>) -> Self {
        FolderSizes { records }
    }
}

/// Holds row values for the storage report.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct ReportItem {
    folder: String,
    size: String,
    percent: f64,
}

impl ReportItem {
    /// Create new ReportItem representing storage in a folder in the Document Center on
    /// CivicEngage.
    pub fn new(folder: &str, size: f64, total_size: f64) -> error::LinkResult<Self> {
        let sz = byte_unit::Byte::from_unit(size, byte_unit::ByteUnit::KB)?;
        let sz = sz.get_appropriate_unit(false);
        let flt = size as f64;
        let pct = flt.div(total_size);
        Ok(ReportItem {
            folder: folder.to_owned(),
            size: sz.to_string(),
            percent: pct,
        })
    }
}

/// Holds a vector of [`ReportItem`] objects.
#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct ReportItems {
    records: Vec<ReportItem>,
}

impl ReportItems {
    /// Outputs a storage report to csv at path `title`.
    pub fn to_csv<P: AsRef<std::path::Path>>(&mut self, title: P) -> Result<(), std::io::Error> {
        utils::to_csv(&mut self.records, title)?;
        Ok(())
    }
}

impl TryFrom<FolderSizes> for ReportItems {
    type Error = error::LinkError;

    fn try_from(folder_sizes: FolderSizes) -> Result<Self, Self::Error> {
        let mut sizes = folder_sizes
            .records_ref()
            .iter()
            .map(|v| v.size())
            .collect::<Vec<f64>>();
        sizes.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let max_size = sizes[sizes.len() - 1];

        let mut records = Vec::new();
        folder_sizes
            .records_ref()
            .iter()
            .map(|item| records.push(ReportItem::new(&item.folder, item.size(), max_size).unwrap()))
            .for_each(drop);
        Ok(ReportItems { records })
    }
}
