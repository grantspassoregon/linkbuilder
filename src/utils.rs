use serde::Serialize;

/// Generic function to write a struct out to a csv file.  Called by internal library functions.
pub fn to_csv<T: Serialize + Clone, P: AsRef<std::path::Path>>(
    item: &mut Vec<T>,
    title: P,
) -> Result<(), std::io::Error> {
    let mut wtr = csv::Writer::from_path(title)?;
    for i in item.clone() {
        wtr.serialize(i)?;
    }
    wtr.flush()?;
    Ok(())
}
