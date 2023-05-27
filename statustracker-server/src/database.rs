use std::{sync::Arc, time::SystemTime};

use color_eyre::eyre::Result;
use futures::StreamExt;
use itertools::Itertools;
use mongodb::{
    bson::{doc, to_bson},
    options::UpdateOptions,
    Database,
};
use rayon::prelude::*;
use tracing::info;

use crate::{
    hour::{AbsRecord, Hour, HourDef, RollingAvgRecord},
    tracker::NameMapWrapper,
    utils::{HourTimestamp, MinuteTimestamp},
};

pub struct STDatabase(pub Database);

impl STDatabase {
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

    #[tracing::instrument(skip(self))]
    pub async fn save_name_map(&self, name_map: &NameMapWrapper) -> Result<()> {
        info!("Saving name_map");
        let mut b = to_bson(name_map)?;
        b.as_document_mut().unwrap().remove("_id");
        self.0
            .collection::<NameMapWrapper>("name_map")
            .update_one(
                doc! {"_id": 0u32},
                doc! {"$set": b},
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await?;
        Ok(())
    }
    pub async fn get_hours(&self, from: HourTimestamp, to: HourTimestamp) -> Result<Vec<Hour>> {
        let a = self
            .0
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
            .map_ok(Into::into)
            .collect::<mongodb::error::Result<Vec<_>>>()?;
        Ok(a)
    }
    pub async fn get_hour(&self, timestamp: HourTimestamp) -> Result<Option<Hour>> {
        Ok(self
            .0
            .collection::<HourDef>("hours")
            .find_one(doc! {"_id": timestamp}, None)
            .await
            .map(|a| a.map(Into::into))?)
    }
    #[allow(clippy::cast_lossless)]
    pub async fn get_minutes(
        &self,
        from: MinuteTimestamp,
        to: MinuteTimestamp,
    ) -> Result<Vec<Option<Arc<AbsRecord>>>> {
        let from_h = (from / 60) as u32;
        let to_h = (to / 60) as u32;
        let mut hours = self.get_hours(from_h, to_h).await?;
        for i in from_h..=to_h {
            if !hours.iter().any(|a| a._id == i) {
                hours.push(Hour::new(i));
            }
        }
        let records = hours
            .iter()
            .sorted_by_cached_key(|a| a._id)
            .flat_map(|h| &h.records)
            .skip((from - from_h as u64 * 60) as usize)
            .take((to - from + 1) as usize)
            .cloned()
            .collect::<Vec<_>>();
        Ok(records)
    }
    pub async fn get_rolling_avg(
        &self,
        from: MinuteTimestamp,
        to: MinuteTimestamp,
        delta: u64,
    ) -> Result<Vec<Option<RollingAvgRecord>>> {
        let mins = self.get_minutes(from - delta, to + delta).await?;
        let udelta = delta as usize;
        Ok((udelta..mins.len() - udelta)
            .into_par_iter()
            .map(|i| {
                mins[i - udelta..=i + udelta]
                    .iter()
                    .filter_map(Option::as_ref)
                    .map(|a| (**a).to_owned())
                    .collect::<Vec<_>>()
            })
            .map(|a| {
                if a.is_empty() {
                    None
                } else {
                    Some((*a).into())
                }
            })
            .collect())
    }
    #[tracing::instrument(skip(self))]
    pub async fn save_hour(&self, hour: Hour) -> Result<()> {
        info!("Saving hour");
        let mut b = to_bson(&HourDef::from(hour.to_owned()))?;
        b.as_document_mut().unwrap().remove("_id");
        self.0
            .collection::<HourDef>("hours")
            .update_one(
                doc! {"_id": hour._id},
                doc! {"$set": b},
                Some(UpdateOptions::builder().upsert(true).build()),
            )
            .await?;
        Ok(())
    }
}
