use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use once_cell::sync::Lazy;
use reqwest::StatusCode;
use serde_json::{Map, Value};
use smol_str::SmolStr;
use tokio::sync::RwLock;
use tracing::{debug, trace};
use uuid::Uuid;

pub static NAME_CACHE: Lazy<RwLock<HashMap<SmolStr, Uuid>>> = Lazy::new(Default::default);

#[tracing::instrument]
pub async fn name_to_uuid(name: &str) -> Result<Option<Uuid>> {
    let mut cache = NAME_CACHE.write().await;
    Ok(if let Some(id) = cache.get(name) {
        debug!(?name, "Retrieving uuid from cache");
        Some(*id)
    } else {
        debug!(?name, "Retrieving uuid from API");
        let req = reqwest::get(format!(
            "https://api.mojang.com/users/profiles/minecraft/{name}"
        ))
        .await?;
        if req.status() == StatusCode::BAD_REQUEST {
            return Ok(None);
        }
        let json: Map<String, Value> = req.json().await?;
        trace!(?name, ?json);
        let id = json
            .get("id")
            .ok_or_else(|| eyre!("No field `id`"))?
            .as_str()
            .ok_or_else(|| eyre!("Field `id` is not string"))?
            .parse::<Uuid>()?;
        cache.insert(name.into(), id);
        Some(id)
    })
}
