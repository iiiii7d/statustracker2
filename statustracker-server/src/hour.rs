use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use smol_str::SmolStr;

use crate::utils::{BitField64, Category, HourTimestamp};

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct RollingAvgRecord {
    pub all: f32,
    pub categories: HashMap<Category, f32>,
}
impl From<AbsRecord> for RollingAvgRecord {
    fn from(value: AbsRecord) -> Self {
        Self {
            all: value.all.len() as f32,
            categories: value
                .categories
                .into_iter()
                .map(|(a, b)| (a, b.len() as f32))
                .collect(),
        }
    }
}
impl From<&[AbsRecord]> for RollingAvgRecord {
    fn from(value: &[AbsRecord]) -> Self {
        Self {
            all: value.into_iter().map(|a| a.all.len() as f32).sum::<f32>() / value.len() as f32,
            categories: {
                let mut counts: HashMap<Category, Vec<f32>> = HashMap::new();
                for a in value {
                    for (cat, s) in &a.categories {
                        counts
                            .entry(cat.to_owned())
                            .or_default()
                            .push(s.len() as f32)
                    }
                }
                let max = counts.values().map(|a| a.len()).max();
                if let Some(max) = max {
                    for l in counts.values_mut() {
                        l.extend(vec![0.0; max - l.len()])
                    }
                }
                counts
                    .into_iter()
                    .map(|(cat, l)| (cat, l.iter().sum::<f32>() / l.len() as f32))
                    .collect()
            },
        }
    }
}

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
    #[must_use]
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

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    use crate::{
        hour::{AbsRecord, Hour, HourDef, Record},
        utils::BitField64,
    };

    #[test]
    pub fn hour_def_to_hour() {
        let hd = HourDef {
            _id: 0,
            tracked_mins: {
                let mut b = BitField64::default();
                b.turn_on(0);
                b.turn_on(1);
                b.turn_on(3);
                b
            },
            deltas: {
                let mut d = HashMap::new();
                d.insert(
                    "0".into(),
                    Record::Abs(AbsRecord {
                        all: HashSet::from([0]),
                        categories: HashMap::default(),
                    }),
                );
                d.insert(
                    "1".into(),
                    Record::Delta {
                        joined: HashSet::from([1]),
                        joined_categories: HashMap::default(),
                        left: HashSet::from([0]),
                        left_categories: HashMap::default(),
                    },
                );
                d.insert(
                    "3".into(),
                    Record::Abs(AbsRecord {
                        all: HashSet::from([1, 2]),
                        categories: HashMap::default(),
                    }),
                );
                d
            },
        };
        let d = Hour::from(hd);
        assert_eq!(
            d.records[0].as_ref().map(|a| (**a).to_owned()),
            Some(AbsRecord {
                all: HashSet::from([0]),
                categories: HashMap::default(),
            })
        );
        assert_eq!(
            d.records[1].as_ref().map(|a| (**a).to_owned()),
            Some(AbsRecord {
                all: HashSet::from([1]),
                categories: HashMap::default(),
            })
        );
        assert_eq!(d.records[2].as_ref().map(|a| (**a).to_owned()), None);
        assert_eq!(
            d.records[3].as_ref().map(|a| (**a).to_owned()),
            Some(AbsRecord {
                all: HashSet::from([1, 2]),
                categories: HashMap::default(),
            })
        );
    }
    #[test]
    pub fn hour_to_hour_def() {
        let h = Hour {
            _id: 0,
            records: {
                let mut r = [(); 60].map(|_| None);
                r[0] = Some(Arc::new(AbsRecord {
                    all: HashSet::from([0]),
                    categories: HashMap::default(),
                }));
                r[1] = Some(Arc::new(AbsRecord {
                    all: HashSet::from([1]),
                    categories: HashMap::default(),
                }));
                r[3] = Some(Arc::new(AbsRecord {
                    all: HashSet::from([1, 2]),
                    categories: HashMap::default(),
                }));
                r
            },
        };
        let hd = HourDef::from(h);
        assert_eq!(
            hd,
            HourDef {
                _id: 0,
                tracked_mins: {
                    let mut b = BitField64::default();
                    b.turn_on(0);
                    b.turn_on(1);
                    b.turn_on(3);
                    b
                },
                deltas: {
                    let mut d = HashMap::new();
                    d.insert(
                        "0".into(),
                        Record::Abs(AbsRecord {
                            all: HashSet::from([0]),
                            categories: HashMap::default(),
                        }),
                    );
                    d.insert(
                        "1".into(),
                        Record::Delta {
                            joined: HashSet::from([1]),
                            joined_categories: HashMap::default(),
                            left: HashSet::from([0]),
                            left_categories: HashMap::default(),
                        },
                    );
                    d.insert(
                        "3".into(),
                        Record::Abs(AbsRecord {
                            all: HashSet::from([1, 2]),
                            categories: HashMap::default(),
                        }),
                    );
                    d
                },
            }
        );
    }
}
