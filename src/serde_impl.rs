//! Serde support: XUIDs serialize as their string representation.

use crate::Xuid;
use core::fmt;
use serde::de::{self, Visitor};
use serde::{Deserializer, Serializer};

impl serde::Serialize for Xuid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

struct XuidVisitor;

impl<'de> Visitor<'de> for XuidVisitor {
    type Value = Xuid;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an XUID or UUID string")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        v.parse().map_err(de::Error::custom)
    }
}

impl<'de> serde::Deserialize<'de> for Xuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(XuidVisitor)
    }
}
