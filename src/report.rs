use crate::{error, utils};
use serde::{Deserialize, Serialize};
use std::ops::Div;

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FolderSize {
    folder: String,
    size: f64,
}

impl FolderSize {
    pub fn size(&self) -> f64 {
        self.size
    }
}

impl FolderSize {
    pub fn new(folder: &str, size: f64) -> Self {
        FolderSize {
            folder: folder.to_owned(),
            size,
        }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct FolderSizes {
    records: Vec<FolderSize>,
}

impl FolderSizes {
    pub fn records(&self) -> Vec<FolderSize> {
        self.records.clone()
    }

    pub fn records_ref(&self) -> &Vec<FolderSize> {
        &self.records
    }

    pub fn size(&self) -> f64 {
        self.records_ref().iter().fold(0.0, |acc, x| acc + x.size())
    }
}

impl From<Vec<FolderSize>> for FolderSizes {
    fn from(records: Vec<FolderSize>) -> Self {
        FolderSizes { records }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct ReportItem {
    folder: String,
    size: String,
    percent: f64,
}

impl ReportItem {
    pub fn new(folder: &str, size: f64, total_size: f64) -> Result<Self, error::LinkError> {
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

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct ReportItems {
    records: Vec<ReportItem>,
}

impl ReportItems {
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
