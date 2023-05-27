use std::{collections::HashMap, time::SystemTime};

use color_eyre::eyre::{eyre, Result};
use futures::StreamExt;
use itertools::Itertools;
use mongodb::{
    bson::{doc, to_bson},
    options::{ClientOptions, UpdateOptions},
    Client, Database,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smol_str::SmolStr;
use tracing::{debug, info, trace};
use uuid::{Bytes, Uuid};

use crate::{
    hour::{AbsRecord, Hour},
    name_to_uuid::name_to_uuid,
    utils::{Category, HourTimestamp, MinuteTimestamp},
};

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub categories: HashMap<Category, Vec<Uuid>>,
    pub dynmap_link: Url,
    pub mongodb_uri: SmolStr,
    pub database_name: SmolStr,
    #[serde(default)]
    pub hosted_over_http: bool,
}

pub struct StatusTracker {
    pub config: Config,
    pub name_map: NameMapWrapper,
    pub database: Database,
}

impl StatusTracker {
    #[tracing::instrument(skip_all)]
    pub async fn new(config: Config) -> Result<Self> {
        debug!("Checking for `all` category");
        if config.categories.keys().contains::<Category>(&"all".into()) {
            return Err(eyre!("Category named `all` found"));
        }
        info!("Creating client");
        let client = Client::with_options(
            ClientOptions::parse(std::env::var(config.mongodb_uri.to_string())?).await?,
        )?;
        let database = client.database(&config.database_name);
        info!("Retrieving name_map");
        let name_map = database
            .collection("name_map")
            .find_one(doc! {"_id": 0u32}, None)
            .await?
            .unwrap_or_default();
        Ok(Self {
            config,
            name_map,
            database,
        })
    }
    pub async fn get_hours(&self, from: HourTimestamp, to: HourTimestamp) -> Result<Vec<Hour>> {
        let a = self
            .database
            .collection::<Hour>("hours")
            .find(
                doc! {
                    "_id": {
                        "$gte": from,
                        "$lte": to
                    }
                },
                None,
            )
            .await?
            .collect::<Vec<_>>()
            .await;
        let a = a.into_iter().collect::<mongodb::error::Result<Vec<_>>>()?;
        Ok(a)
    }
    pub async fn get_hour(&self, timestamp: HourTimestamp) -> Result<Option<Hour>> {
        Ok(self
            .database
            .collection("hours")
            .find_one(doc! {"_id": timestamp}, None)
            .await?)
    }
    #[tracing::instrument(skip(self))]
    pub async fn save_hour(&self, hour: Hour) -> Result<()> {
        info!("Saving hour");
        let mut b = to_bson(&hour)?;
        b.as_document_mut().unwrap().remove("_id");
        self.database
            .collection::<Hour>("hours")
            .update_one(
                doc! {"_id": hour._id},
                doc! {"$set": b},
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await?;
        Ok(())
    }
    #[tracing::instrument(skip(self))]
    pub async fn save_name_map(&self) -> Result<()> {
        info!("Saving name_map");
        let mut b = to_bson(&self.name_map)?;
        b.as_document_mut().unwrap().remove("_id");
        self.database
            .collection::<NameMapWrapper>("name_map")
            .update_one(
                doc! {"_id": 0u32},
                doc! {"$set": b},
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await?;
        Ok(())
    }
    #[tracing::instrument(skip(self))]
    pub async fn pull_from_dynmap(&self) -> Result<Vec<SmolStr>> {
        info!("Pulling player list from Dynmap");
        let json: Map<String, Value> = reqwest::get(self.config.dynmap_link.to_owned())
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
            .collect::<Result<Vec<_>, _>>()
    }
    #[tracing::instrument(skip(self))]
    pub async fn update_name_map(&mut self, names: Vec<SmolStr>) -> Result<Vec<(Uuid, usize)>> {
        let mut uuids = vec![];
        for name in names {
            let uuid = name_to_uuid(&name)
                .await?
                .ok_or_else(|| eyre!("Invalid username {name}"))?;
            debug!(%name, "Updating name map");
            let index = self
                .name_map
                .data
                .iter()
                .position(|a| a == uuid.as_bytes())
                .unwrap_or_else(|| {
                    self.name_map.data.push(*uuid.as_bytes());
                    self.name_map.data.len() - 1
                });
            uuids.push((uuid, index));
        }
        Ok(uuids)
    }
    #[tracing::instrument(skip(self))]
    pub fn split_into_categories(&self, ids: Vec<(Uuid, usize)>) -> AbsRecord {
        let mut record = AbsRecord::default();
        for (uuid, id) in ids {
            debug!(%uuid, id, "Splitting into categories");
            record.all.insert(id);
            for (cat, cat_ids) in &self.config.categories {
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
    #[tracing::instrument(skip(self))]
    pub async fn add_record(&self, record: AbsRecord) -> Result<()> {
        let timestamp: MinuteTimestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 60;
        let mut hour = self
            .get_hour((timestamp / 60) as HourTimestamp)
            .await?
            .unwrap_or_else(|| Hour::new(timestamp, record.to_owned()));
        info!(timestamp, "Adding record");
        hour.add_record(timestamp, record);
        self.save_hour(hour).await?;
        Ok(())
    }
    pub async fn run(&mut self) -> Result<()> {
        let names = self.pull_from_dynmap().await?;
        let ids = self.update_name_map(names).await?;
        let record = self.split_into_categories(ids);
        self.add_record(record).await?;
        self.save_name_map().await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct NameMapWrapper {
    pub _id: u32,
    pub data: Vec<Bytes>,
}
