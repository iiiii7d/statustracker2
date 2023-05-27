use std::{sync::Arc, time::SystemTime};

use color_eyre::eyre::{eyre, Result};
use futures::StreamExt;
use itertools::Itertools;
use mongodb::{
    bson::{doc, to_bson},
    options::{ClientOptions, UpdateOptions},
    Client, Database,
};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::{debug, info};
use uuid::{Bytes, Uuid};

use crate::{
    config::Config,
    hour::{AbsRecord, Hour, HourDef},
    name_to_uuid::name_to_uuid,
    utils::{Category, HourTimestamp, MinuteTimestamp},
};

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
            .collection::<HourDef>("hours")
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
        let a = a
            .into_iter()
            .map_ok(std::convert::Into::into)
            .collect::<mongodb::error::Result<Vec<_>>>()?;
        Ok(a)
    }
    pub async fn get_hour(&self, timestamp: HourTimestamp) -> Result<Option<Hour>> {
        Ok(self
            .database
            .collection::<HourDef>("hours")
            .find_one(doc! {"_id": timestamp}, None)
            .await
            .map(|a| a.map(std::convert::Into::into))?)
    }
    #[tracing::instrument(skip(self))]
    pub async fn save_hour(&self, hour: Hour) -> Result<()> {
        info!("Saving hour");
        let mut b = to_bson(&HourDef::from(hour.to_owned()))?;
        b.as_document_mut().unwrap().remove("_id");
        self.database
            .collection::<HourDef>("hours")
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
    pub async fn add_record(&self, record: AbsRecord) -> Result<()> {
        let min_ts: MinuteTimestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 60;
        let h_ts = (min_ts / 60) as HourTimestamp;
        let mut hour = self
            .get_hour(h_ts)
            .await?
            .unwrap_or_else(|| Hour::new(h_ts));
        info!(min_ts, "Adding record");
        hour.records[(min_ts - u64::from(h_ts) * 60) as usize] = Some(Arc::new(record));
        self.save_hour(hour).await?;
        Ok(())
    }
    pub async fn run(&mut self) -> Result<()> {
        let names = self.config.pull_from_dynmap().await?;
        let ids = self.name_map.update_name_map(names).await?;
        let record = self.config.split_into_categories(ids);
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
impl NameMapWrapper {
    #[tracing::instrument(skip(self))]
    pub async fn update_name_map(&mut self, names: Vec<SmolStr>) -> Result<Vec<(Uuid, usize)>> {
        let mut uuids = vec![];
        for name in names {
            let uuid = name_to_uuid(&name)
                .await?
                .ok_or_else(|| eyre!("Invalid username {name}"))?;
            debug!(%name, "Updating name map");
            let index = self
                .data
                .iter()
                .position(|a| a == uuid.as_bytes())
                .unwrap_or_else(|| {
                    self.data.push(*uuid.as_bytes());
                    self.data.len() - 1
                });
            uuids.push((uuid, index));
        }
        Ok(uuids)
    }
}
