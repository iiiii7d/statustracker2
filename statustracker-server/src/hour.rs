use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::{debug, trace};

use crate::utils::{BitField64, Category, HourTimestamp, MinuteTimestamp};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Abs;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Join;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Leave;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Record<T> {
    pub all: HashSet<usize>,
    pub categories: HashMap<Category, HashSet<usize>>,
    #[serde(skip)]
    marker: PhantomData<T>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct DeltaRecord {
    pub joins: Option<Record<Join>>,
    pub leaves: Option<Record<Leave>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Hour {
    pub _id: HourTimestamp,
    pub init: Record<Abs>,
    pub start_min: u8,
    pub tracked_mins: BitField64,
    pub deltas: HashMap<SmolStr, DeltaRecord>,
}
impl Hour {
    pub fn new(timestamp: MinuteTimestamp, init: Record<Abs>) -> Self {
        Self {
            _id: (timestamp / 60) as u32,
            init,
            start_min: (timestamp - timestamp / 60 * 60) as u8,
            ..Default::default()
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn calculate_abs_record(&self, minute_no: u8) -> Record<Abs> {
        let mut record = self.init.to_owned();
        let empty_hash_set = HashSet::new();
        for (m, delta) in self
            .deltas
            .iter()
            .sorted_by_key(|(k, _)| k.parse::<u8>().unwrap())
        {
            let m = m.parse::<u8>().unwrap();
            debug!(m, "Calculating absolute time record");
            if m > minute_no {
                break;
            }
            record.all = record
                .all
                .union(
                    delta
                        .joins
                        .as_ref()
                        .map(|a| &a.all)
                        .unwrap_or(&empty_hash_set),
                )
                .cloned()
                .collect();
            record.all = record
                .all
                .difference(
                    delta
                        .leaves
                        .as_ref()
                        .map(|a| &a.all)
                        .unwrap_or(&empty_hash_set),
                )
                .cloned()
                .collect();
            for (cat, list) in &mut record.categories {
                *list = list
                    .union(
                        delta
                            .joins
                            .as_ref()
                            .and_then(|a| a.categories.get(cat))
                            .unwrap_or(&empty_hash_set),
                    )
                    .cloned()
                    .collect();
                *list = list
                    .difference(
                        delta
                            .leaves
                            .as_ref()
                            .and_then(|a| a.categories.get(cat))
                            .unwrap_or(&empty_hash_set),
                    )
                    .cloned()
                    .collect();
            }
        }
        trace!(?record);
        record
    }
    #[tracing::instrument(skip(self, record))]
    pub fn add_delta(&mut self, timestamp: MinuteTimestamp, record: Record<Abs>) {
        let minute_no = (timestamp - self._id as u64 * 60) as u8;
        if minute_no >= 60 {
            panic!("{minute_no}");
        }
        self.tracked_mins.turn_on(minute_no as usize);
        let empty_hash_set = HashSet::new();
        let m = if let Some((m, _)) = self
            .deltas
            .iter()
            .max_by_key(|(k, _)| k.parse::<u8>().unwrap())
        {
            m.parse::<u8>().unwrap()
        } else {
            0
        };
        debug!(last_minute = m);
        let latest_abs = self.calculate_abs_record(m);
        trace!(?latest_abs);
        if latest_abs == record {
            debug!("No change found");
            return;
        }
        let mut delta = DeltaRecord {
            joins: Some(Record {
                all: record.all.difference(&latest_abs.all).cloned().collect(),
                categories: record
                    .categories
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.to_owned(),
                            v.difference(latest_abs.categories.get(k).unwrap_or(&empty_hash_set))
                                .cloned()
                                .collect::<HashSet<_>>(),
                        )
                    })
                    .filter(|(_, v)| !v.is_empty())
                    .collect(),
                marker: Default::default(),
            }),
            leaves: Some(Record {
                all: latest_abs.all.difference(&record.all).cloned().collect(),
                categories: record
                    .categories
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.to_owned(),
                            latest_abs
                                .categories
                                .get(k)
                                .unwrap_or(&empty_hash_set)
                                .difference(v)
                                .cloned()
                                .collect::<HashSet<_>>(),
                        )
                    })
                    .filter(|(_, v)| !v.is_empty())
                    .collect(),
                marker: Default::default(),
            }),
        };
        if delta.joins == Some(Default::default()) {
            delta.joins = None;
        }
        if delta.joins == Some(Default::default()) {
            delta.joins = None;
        }
        trace!(?delta);
        self.deltas.insert(minute_no.to_string().into(), delta);
    }
}
