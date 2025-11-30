use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, Visitor},
};
use time::UtcDateTime;

/// A generic [Discord Snowflake ID](https://discord.com/developers/docs/reference#snowflakes).
///
/// Prefer using the typed wrappers to prevent mixing e.g. user IDs with emoji IDs.
///
/// [`Snowflake`]s are [`Ord`] comparing their timestamp (equal timestamps compare "internal worker
/// id", then "internal process id" and finally "increment").
///
/// [`Snowflake`]s can be [`Deserialize`]d not only from [`u64`] but also from strings. Discord
/// usually returns [`Snowflake`]s as strings since some API users don't have a good way to parse
/// [`u64`] in their json.
///
/// They are however always [`Serialize`]d as integers, since Discord itself should be able to deal
/// with that without issue.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Snowflake(pub u64);

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SnowflakeVisitor)
    }
}

impl Snowflake {
    /// The number of milliseconds since the Discord epoch, i.e. the start of 2015.
    const DISCORD_EPOCH_MS: u64 = 1420070400000;

    /// The timestamp that is encoded in this [`Snowflake`].
    ///
    /// Can range from `2015-01-01 0:00:00.0` to `2154-05-15 7:35:11.103` with millisecond accuracy.
    pub fn timestamp(self) -> UtcDateTime {
        let unix_timestamp_ms = (self.0 >> 22) + Self::DISCORD_EPOCH_MS;
        UtcDateTime::from_unix_timestamp_nanos(unix_timestamp_ms as i128 * 1_000_000)
            .expect("Snowflake should have a smaller range than UtcDateTime")
    }

    /// The "internal worker ID".
    pub fn internal_worker_id(self) -> u8 {
        (self.0 >> 17) as u8 & 0b11111
    }

    /// The "internal process ID".
    pub fn internal_process_id(self) -> u8 {
        (self.0 >> 12) as u8 & 0b11111
    }

    /// For every ID that is generated on that process, this number is incremented.
    pub fn increment(self) -> u16 {
        self.0 as u16 & 0xFFF
    }
}

struct SnowflakeVisitor;

impl<'de> Visitor<'de> for SnowflakeVisitor {
    type Value = Snowflake;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a Discord Snowflake (u64 or string)")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Snowflake(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Snowflake(v.parse().map_err(de::Error::custom)?))
    }
}
