use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use tracing::{debug, trace};

use crate::utils::{BitField64, Category, HourTimestamp, MinuteTimestamp};

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

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Hour {
    pub _id: HourTimestamp,
    pub tracked_mins: BitField64,
    pub deltas: HashMap<SmolStr, Record>,
}
impl Hour {
    #[must_use]
    pub fn new(timestamp: MinuteTimestamp, abs: AbsRecord) -> Self {
        let min = timestamp - timestamp / 60 * 60;
        Self {
            _id: (timestamp / 60) as u32,
            tracked_mins: {
                let mut bits = BitField64::default();
                bits.turn_on(min as i32);
                bits
            },
            deltas: HashMap::from([(min.to_string().into(), Record::Abs(abs))]),
        }
    }
    #[allow(clippy::unwrap_in_result)]
    #[tracing::instrument(skip(self))]
    pub fn calculate_abs_record(&self, minute_no: u8) -> Option<AbsRecord> {
        let mut abs_record = None;
        for (m, record) in self
            .deltas
            .iter()
            .sorted_by_key(|(k, _)| k.parse::<u8>().unwrap())
        {
            let m = m.parse::<u8>().unwrap();
            debug!(m, "Calculating absolute time record");
            if m > minute_no {
                break;
            }
            match record {
                Record::Abs(record) => {
                    abs_record = Some(record.to_owned());
                }
                Record::Delta {
                    joined,
                    joined_categories,
                    left,
                    left_categories,
                } => {
                    let Some(abs_record) = &mut abs_record else {
                        continue;
                    };
                    abs_record.all = abs_record.all.union(joined).copied().collect();
                    abs_record.all = abs_record.all.difference(left).copied().collect();
                    for (cat, other_list) in joined_categories {
                        let list = abs_record.categories.entry(cat.to_owned()).or_default();
                        *list = list.union(other_list).copied().collect();
                    }
                    for (cat, other_list) in left_categories {
                        let list = abs_record.categories.entry(cat.to_owned()).or_default();
                        *list = list.difference(other_list).copied().collect();
                    }
                }
            }
        }
        trace!(?abs_record);
        abs_record
    }

    #[tracing::instrument(skip(self, abs))]
    pub fn add_record(&mut self, timestamp: MinuteTimestamp, abs: AbsRecord) {
        let minute_no = (timestamp - u64::from(self._id) * 60) as u8;
        assert!(minute_no < 60, "{minute_no}");
        self.tracked_mins.turn_on(i32::from(minute_no));

        if minute_no == 0 || !self.tracked_mins.is_on(i32::from(minute_no) - 1) {
            debug!(minute_no, "Adding Abs record");
            self.deltas
                .insert(minute_no.to_string().into(), Record::Abs(abs));
            return;
        }

        let latest_abs = self.calculate_abs_record(minute_no).unwrap_or_default();
        trace!(?latest_abs);

        if latest_abs == abs {
            debug!("No change, nothing to add");
            return;
        }
        debug!(minute_no, "Adding Delta record");

        let empty_hash_set = &HashSet::new();

        let delta = Record::Delta {
            joined: abs.all.difference(&latest_abs.all).copied().collect(),
            joined_categories: {
                let mut joined = HashMap::new();
                for cat in abs
                    .categories
                    .keys()
                    .chain(latest_abs.categories.keys())
                    .sorted()
                    .dedup()
                {
                    let joined_cat = abs
                        .categories
                        .get(cat)
                        .unwrap_or(empty_hash_set)
                        .difference(latest_abs.categories.get(cat).unwrap_or(empty_hash_set))
                        .copied()
                        .collect::<HashSet<_>>();
                    if !joined_cat.is_empty() {
                        joined.insert(cat.to_owned(), joined_cat);
                    }
                }
                joined
            },
            left: latest_abs.all.difference(&abs.all).copied().collect(),
            left_categories: {
                let mut left = HashMap::new();
                for cat in abs
                    .categories
                    .keys()
                    .chain(latest_abs.categories.keys())
                    .sorted()
                    .dedup()
                {
                    let left_cat = latest_abs
                        .categories
                        .get(cat)
                        .unwrap_or(empty_hash_set)
                        .difference(abs.categories.get(cat).unwrap_or(empty_hash_set))
                        .copied()
                        .collect::<HashSet<_>>();
                    if !left_cat.is_empty() {
                        left.insert(cat.to_owned(), left_cat);
                    }
                }
                left
            },
        };

        trace!(?delta);
        self.deltas.insert(minute_no.to_string().into(), delta);
    }
}
