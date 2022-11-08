use std::{fmt::Formatter, time::SystemTime};

use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use smol_str::SmolStr;

pub type HourTimestamp = u32;
pub type MinuteTimestamp = u64;
pub type Category = SmolStr;

pub fn get_hour_timestamp(t: SystemTime) -> HourTimestamp {
    (t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() / 60 / 60) as HourTimestamp
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitField60(pub [bool; 60]);
impl Default for BitField60 {
    fn default() -> Self {
        Self([false; 60])
    }
}
impl Serialize for BitField60 {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut n = 0;
        for (i, b) in self.0.into_iter().enumerate() {
            if b {
                n |= 1 << i;
            }
        }
        ser.serialize_i64(n)
    }
}
impl<'de> Deserialize<'de> for BitField60 {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct I64Visitor;
        impl<'de> Visitor<'de> for I64Visitor {
            type Value = i64;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("an i64")
            }

            fn visit_i64<E: Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(v)
            }
        }
        let n = de.deserialize_i64(I64Visitor)?;
        let mut arr = [false; 60];
        for (i, e) in arr.iter_mut().enumerate() {
            if (1 << i) & n != 0 {
                *e = true
            }
        }
        Ok(Self(arr))
    }
}
