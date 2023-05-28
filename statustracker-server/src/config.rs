use std::collections::HashMap;

use color_eyre::eyre::eyre;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smol_str::SmolStr;
use tracing::{debug, info, trace};
use url::Url;
use uuid::Uuid;

use crate::{hour::AbsRecord, utils::Category};

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub categories: HashMap<Category, Vec<Uuid>>,
    pub dynmap_link: Url,
    pub mongodb_uri: SmolStr,
    pub database_name: SmolStr,
    #[serde(default)]
    pub hosted_over_http: bool,
    #[serde(default)]
    pub no_write: bool,
}

impl Config {
    #[tracing::instrument(skip(self))]
    pub async fn pull_from_dynmap(&self) -> color_eyre::Result<Vec<SmolStr>> {
        info!("Pulling player list from Dynmap");
        let json: Map<String, Value> = reqwest::get(self.dynmap_link.to_owned())
            .await?
            .json()
            .await?;
        trace!(?json);
        json.get("players")
            .ok_or_else(|| eyre!("No field `players`"))?
            .as_array()
            .ok_or_else(|| eyre!("Field `players` is not an array"))?
            .iter()
            .map(|o| {
                o.as_object()
                    .ok_or_else(|| eyre!("Elements of field `players` are not objects"))?
                    .get("account")
                    .ok_or_else(|| eyre!("No field `account` in player object"))?
                    .as_str()
                    .ok_or_else(|| eyre!("Field `account` in player object is not string"))
            })
            .map_ok(std::convert::Into::into)
            .collect::<color_eyre::Result<Vec<_>, _>>()
    }

    #[tracing::instrument(skip(self))]
    pub fn split_into_categories(&self, ids: Vec<(Uuid, usize)>) -> AbsRecord {
        let mut record = AbsRecord::default();
        for (uuid, id) in ids {
            debug!(%uuid, id, "Splitting into categories");
            record.all.insert(id);
            for (cat, cat_ids) in &self.categories {
                if cat_ids.contains(&uuid) {
                    record
                        .categories
                        .entry(cat.to_owned())
                        .or_default()
                        .insert(id);
                }
            }
        }
        record
    }
}
