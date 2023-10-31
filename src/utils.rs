use crate::prelude::*;
use serde::Serialize;
use tracing::{info, trace};

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

/// This function authenticates a user with the CivicEngage API.
pub async fn load_user() -> LinkResult<AuthorizedUser> {
    trace!("Loading environmental variables.");
    let api_key = std::env::var("API_KEY")?;
    let partition = std::env::var("PARTITION")?;
    let name = std::env::var("USERNAME")?;
    let password = std::env::var("PASSWORD")?;
    let host = std::env::var("HOST")?;
    trace!("Environmental variables loaded.");

    trace!("Creating user from environmental variables.");
    let user = User::new()
        .api_key(&api_key)
        .partition(&partition)
        .name(&name)
        .password(&password)
        .host(&host)
        .build()?;

    trace!("Preparing authorization headers.");
    let headers = AuthorizeHeaders::default();
    trace!("Authorizing user.");
    let auth_info = AuthorizeInfo::new(&user, headers);
    let url = std::env::var("AUTHENTICATE")?;
    let auth_res = auth_info.authorize(&url).await?;
    info!("Authorization successful for user {}.", &auth_res.id());
    trace!("Recording session id of user.");
    Ok(AuthorizedUser::new(&user, &auth_res))
}
