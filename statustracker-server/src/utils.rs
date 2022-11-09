use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

pub type HourTimestamp = u32;
pub type MinuteTimestamp = u64;
pub type Category = SmolStr;

pub fn get_hour_timestamp(t: SystemTime) -> HourTimestamp {
    (t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() / 60 / 60) as HourTimestamp
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BitField64(pub i64);
impl BitField64 {
    pub fn turn_on(&mut self, i: usize) {
        self.0 |= 1 << i
    }
    pub fn is_on(self, i: usize) -> bool {
        (self.0 & (1 << i)) != 0
    }
}
