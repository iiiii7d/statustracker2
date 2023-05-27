use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use mongodb::{bson::doc, options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::{debug, info};
use uuid::{Bytes, Uuid};

use crate::{config::Config, database::STDatabase, name_to_uuid::name_to_uuid, utils::Category};

pub struct StatusTracker {
    pub config: Config,
    pub name_map: NameMapWrapper,
    pub database: STDatabase,
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
        let database = STDatabase(client.database(&config.database_name));
        info!("Retrieving name_map");
        let name_map = database
            .0
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
    pub async fn run(&mut self) -> Result<()> {
        let names = self.config.pull_from_dynmap().await?;
        let ids = self.name_map.update_name_map(names).await?;
        let record = self.config.split_into_categories(ids);
        self.database.add_record(record).await?;
        self.database.save_name_map(&self.name_map).await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
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
