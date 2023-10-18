use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct Fila {
    #[serde(rename(deserialize = "OID_"))]
    object_id: i64,
    instrument: String,
    // assessment: String,
    // utility: String,
    // doc_link: String,
    #[serde(rename(deserialize = "GlobalID"))]
    global_id: String,
    // created_user: String,
    // created_date: String,
    // last_edited_user: String,
    // last_edited_date: String,
    // notes: String,
}

impl Fila {
    pub fn object_id(&self) -> i64 {
        self.object_id
    }

    pub fn instrument_ref(&self) -> &String {
        &self.instrument
    }

    pub fn instrument(&self) -> String {
        self.instrument.clone()
    }

    pub fn global_id(&self) -> String {
        self.global_id.clone()
    }

    pub fn global_id_ref(&self) -> &String {
        &self.global_id
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Filas {
    records: Vec<Fila>,
}

impl Filas {
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let mut records = Vec::new();
        let file = std::fs::File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);

        for result in rdr.deserialize() {
            let record: Fila = result?;
            records.push(record);
        }

        Ok(Filas { records })
    }

    pub fn records(&self) -> Vec<Fila> {
        self.records.clone()
    }

    pub fn records_ref(&self) -> &Vec<Fila> {
        &self.records
    }
}
