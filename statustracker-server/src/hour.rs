use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use smol_str::SmolStr;

use crate::utils::{BitField64, Category, HourTimestamp};

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct AbsRecord {
    pub all: HashSet<usize>,
    pub categories: HashMap<Category, HashSet<usize>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Record {
    Abs(AbsRecord),
    Delta {
        joined: HashSet<usize>,
        joined_categories: HashMap<Category, HashSet<usize>>,
        left: HashSet<usize>,
        left_categories: HashMap<Category, HashSet<usize>>,
    },
}

impl Default for Record {
    fn default() -> Self {
        Self::Abs(AbsRecord::default())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Hour {
    pub _id: HourTimestamp,
    #[serde(with = "BigArray")]
    pub records: [Option<Arc<AbsRecord>>; 60],
}
impl Hour {
    pub fn new(timestamp: HourTimestamp) -> Self {
        Self {
            _id: timestamp,
            ..Self::default()
        }
    }
}
impl Default for Hour {
    fn default() -> Self {
        Self {
            _id: HourTimestamp::default(),
            records: [(); 60].map(|_| None),
        }
    }
}
impl From<HourDef> for Hour {
    #[allow(clippy::unwrap_in_result)]
    #[tracing::instrument]
    fn from(value: HourDef) -> Self {
        let mut hour = Self::new(value._id);
        let prev_record: fn(usize, &Self) -> Option<Arc<AbsRecord>> = |m, h| {
            if m == 0 {
                None
            } else {
                h.records[m - 1].to_owned()
            }
        };
        for m in 0usize..=59 {
            let record = value.deltas.get(&*m.to_string());
            if !value.tracked_mins.is_on(m.try_into().unwrap()) {
                continue;
            }
            hour.records[m] = match record {
                Some(Record::Abs(record)) => Some(Arc::new(record.to_owned())),
                Some(Record::Delta {
                    joined,
                    joined_categories,
                    left,
                    left_categories,
                }) => {
                    let Some(prev) = prev_record(m, &hour) else {
                        panic!("No abs record to compare to")
                    };
                    let mut prev = (*prev).to_owned();
                    prev.all = prev.all.union(joined).copied().collect();
                    prev.all = prev.all.difference(left).copied().collect();
                    for (cat, other_list) in joined_categories {
                        let list = prev.categories.entry(cat.to_owned()).or_default();
                        *list = list.union(other_list).copied().collect();
                    }
                    for (cat, other_list) in left_categories {
                        let list = prev.categories.entry(cat.to_owned()).or_default();
                        *list = list.difference(other_list).copied().collect();
                    }
                    Some(Arc::new(prev))
                }
                None => prev_record(m, &hour).map(|prev| Arc::clone(&prev)),
            }
        }
        hour
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct HourDef {
    pub _id: HourTimestamp,
    pub tracked_mins: BitField64,
    pub deltas: HashMap<SmolStr, Record>,
}
impl From<Hour> for HourDef {
    fn from(value: Hour) -> Self {
        let mut hour = Self {
            _id: value._id,
            ..Self::default()
        };
        let empty_hash_set = &HashSet::new();
        let mut prev_record: Option<Arc<AbsRecord>> = None;

        for (i, record) in value.records.into_iter().enumerate() {
            let Some(record) = record else {
                prev_record = None;
                continue
            };

            hour.tracked_mins.turn_on(i.try_into().unwrap());

            if i == 0 || prev_record.is_none() {
                hour.deltas
                    .insert(i.to_string().into(), Record::Abs((*record).to_owned()));
            } else if let Some(prev) = prev_record {
                let delta = Record::Delta {
                    joined: record.all.difference(&prev.all).copied().collect(),
                    joined_categories: {
                        let mut joined = HashMap::new();
                        for cat in record
                            .categories
                            .keys()
                            .chain(prev.categories.keys())
                            .sorted()
                            .dedup()
                        {
                            let joined_cat = record
                                .categories
                                .get(cat)
                                .unwrap_or(empty_hash_set)
                                .difference(prev.categories.get(cat).unwrap_or(empty_hash_set))
                                .copied()
                                .collect::<HashSet<_>>();
                            if !joined_cat.is_empty() {
                                joined.insert(cat.to_owned(), joined_cat);
                            }
                        }
                        joined
                    },
                    left: prev.all.difference(&record.all).copied().collect(),
                    left_categories: {
                        let mut left = HashMap::new();
                        for cat in record
                            .categories
                            .keys()
                            .chain(prev.categories.keys())
                            .sorted()
                            .dedup()
                        {
                            let left_cat = prev
                                .categories
                                .get(cat)
                                .unwrap_or(empty_hash_set)
                                .difference(record.categories.get(cat).unwrap_or(empty_hash_set))
                                .copied()
                                .collect::<HashSet<_>>();
                            if !left_cat.is_empty() {
                                left.insert(cat.to_owned(), left_cat);
                            }
                        }
                        left
                    },
                };
                hour.deltas.insert(i.to_string().into(), delta);
            }
            prev_record = Some(Arc::clone(&record));
        }
        hour
    }
}
