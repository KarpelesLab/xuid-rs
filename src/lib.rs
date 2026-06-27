//! # XUID
//!
//! XUID (eXtended Unique IDentifier) enhances UUIDs with a short type prefix while
//! keeping the standard 128-bit UUID size. XUIDs are encoded in base32 (rather than
//! base16), so they never exceed 36 characters and are more human-readable.
//!
//! This is a Rust port of the Go library [`KarpelesLab/xuid`]. The string format is
//! byte-for-byte compatible: an XUID produced by either implementation parses cleanly
//! in the other.
//!
//! ```
//! use xuid::Xuid;
//!
//! // Time-ordered (UUIDv7) by default — sortable and index-friendly.
//! let id = Xuid::new("user");
//! println!("{id}"); // e.g. user-h4nu2n-zu3f-dmnn-kguv-6f643nei
//!
//! // Parse back, including from a plain UUID string.
//! let parsed: Xuid = "user-h4nu2n-zu3f-dmnn-kguv-6f643nei".parse().unwrap();
//! assert_eq!(parsed.prefix(), "user");
//!
//! // Deterministic IDs from a key.
//! let a = Xuid::from_key_prefix("specific-resource-name", "res");
//! let b = Xuid::from_key_prefix("specific-resource-name", "res");
//! assert_eq!(a, b);
//! ```
//!
//! [`KarpelesLab/xuid`]: https://github.com/KarpelesLab/xuid

use core::fmt;
use core::str::FromStr;

use data_encoding::BASE32_NOPAD;
use uuid::Uuid;

mod error;
pub use error::Error;

#[cfg(feature = "serde")]
mod serde_impl;

#[cfg(feature = "sqlx")]
mod sqlx_impl;

#[cfg(feature = "rusqlite")]
mod rusqlite_impl;

/// Reference namespace used to derive deterministic XUIDs from keys (matches the Go lib).
const REF_NS: Uuid = Uuid::from_bytes([
    0xd1, 0x6b, 0x61, 0x39, 0x89, 0x89, 0x46, 0x7f, 0xa2, 0x40, 0x44, 0x1d, 0xf6, 0x73, 0x4f, 0x45,
]);

/// Maximum number of prefix characters kept in the string representation.
const MAX_PREFIX: usize = 5;

/// An extended UUID: a standard [`Uuid`] paired with an optional type prefix.
///
/// The prefix is an arbitrary string, but only its first [`MAX_PREFIX`](self) characters
/// appear in the string representation, where they are also lowercased. The prefix is
/// stored verbatim, so two XUIDs whose prefixes differ only in case are **not** equal
/// even though they render identically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Xuid {
    prefix: String,
    uuid: Uuid,
}

impl Xuid {
    /// Creates a new time-ordered (UUIDv7) XUID with the given prefix.
    ///
    /// This is the recommended constructor: the embedded timestamp makes the IDs
    /// sortable by creation time and friendly to database indexes.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self::new_v7(prefix)
    }

    /// Creates a new time-ordered (UUIDv7) XUID with the given prefix.
    pub fn new_v7(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            uuid: Uuid::now_v7(),
        }
    }

    /// Creates a new fully random (UUIDv4) XUID with the given prefix.
    pub fn new_random(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            uuid: Uuid::new_v4(),
        }
    }

    /// Builds an XUID from an existing [`Uuid`] and a prefix.
    pub fn from_uuid(uuid: Uuid, prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            uuid,
        }
    }

    /// Derives a deterministic XUID from `key`, always using the `"utref"` prefix.
    ///
    /// The same key always yields the same XUID (UUIDv5 / SHA-1 over a fixed namespace).
    pub fn from_key(key: impl AsRef<[u8]>) -> Self {
        Self::from_uuid(Uuid::new_v5(&REF_NS, key.as_ref()), "utref")
    }

    /// Derives a deterministic XUID from both `key` and `prefix`.
    ///
    /// The prefix seeds a per-prefix namespace, so the same `(key, prefix)` pair always
    /// produces the same XUID and different prefixes never collide.
    pub fn from_key_prefix(key: impl AsRef<[u8]>, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        let sub_ns = Uuid::new_v5(&REF_NS, prefix.as_bytes());
        Self::from_uuid(Uuid::new_v5(&sub_ns, key.as_ref()), prefix)
    }

    /// Parses `s` and verifies the prefix matches `prefix`, returning [`Error::BadPrefix`]
    /// otherwise. Useful for type-safe parsing of a known entity type.
    pub fn parse_prefix(s: &str, prefix: &str) -> Result<Self, Error> {
        let x = s.parse::<Self>()?;
        if x.prefix != prefix {
            return Err(Error::BadPrefix {
                expected: prefix.to_owned(),
                found: x.prefix,
            });
        }
        Ok(x)
    }

    /// Parses a standard UUID string into an XUID with the given prefix.
    pub fn parse_uuid(uuid: &str, prefix: impl Into<String>) -> Result<Self, Error> {
        Ok(Self::from_uuid(Uuid::parse_str(uuid)?, prefix))
    }

    /// Returns the type prefix (stored verbatim, not truncated or lowercased).
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Returns the underlying [`Uuid`].
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Returns the underlying UUID in canonical hyphenated form, e.g.
    /// `3f1b4d37-34d9-46c6-b546-a57c5f736d22`.
    pub fn to_uuid_string(&self) -> String {
        self.uuid.to_string()
    }
}

impl fmt::Display for Xuid {
    /// Formats as `prefix-aaaaaa-aaaa-aaaa-aaaa-aaaaaaaa` (the prefix is omitted when empty).
    ///
    /// Output is always lowercase, and the prefix is clipped to the first
    /// [`MAX_PREFIX`](self) characters.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 16 bytes -> 26 base32 chars (no padding), lowercased to match the Go encoder.
        let enc = BASE32_NOPAD
            .encode(self.uuid.as_bytes())
            .to_ascii_lowercase();
        let b = enc.as_bytes();

        if !self.prefix.is_empty() {
            let pfx: String = self.prefix.chars().take(MAX_PREFIX).collect();
            f.write_str(&pfx.to_lowercase())?;
            f.write_str("-")?;
        }

        // Hyphenate in the same positions as a regular UUID: 6-4-4-4-8.
        f.write_str(core::str::from_utf8(&b[0..6]).unwrap())?;
        f.write_str("-")?;
        f.write_str(core::str::from_utf8(&b[6..10]).unwrap())?;
        f.write_str("-")?;
        f.write_str(core::str::from_utf8(&b[10..14]).unwrap())?;
        f.write_str("-")?;
        f.write_str(core::str::from_utf8(&b[14..18]).unwrap())?;
        f.write_str("-")?;
        f.write_str(core::str::from_utf8(&b[18..26]).unwrap())?;
        Ok(())
    }
}

impl FromStr for Xuid {
    type Err = Error;

    /// Parses an XUID, or falls back to interpreting `s` as a plain UUID (empty prefix).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Work on bytes so that arbitrary (even non-ASCII) input can never panic a
        // str-slice on a char boundary; we fall back to UUID parsing instead.
        let sb = s.as_bytes();
        // Body length is 30 (6-4-4-4-8 with four hyphens). Total length is therefore
        // 30 (no prefix) or 32..=36 (1..=5 char prefix + '-' + body).
        let (prefix_bytes, body) = match sb.len() {
            30 => (&b""[..], sb),
            32..=36 => {
                let pfx_len = sb.len() - 31;
                if sb[pfx_len] != b'-' {
                    return Self::parse_uuid(s, "");
                }
                (&sb[..pfx_len], &sb[pfx_len + 1..])
            }
            _ => return Self::parse_uuid(s, ""),
        };

        // `body` is now exactly 30 bytes; verify the hyphen layout.
        if body[6] != b'-' || body[11] != b'-' || body[16] != b'-' || body[21] != b'-' {
            return Self::parse_uuid(s, "");
        }

        // Strip the hyphens into the 26 base32 chars and uppercase (the alphabet is uppercase).
        let mut raw = [0u8; 26];
        raw[0..6].copy_from_slice(&body[0..6]);
        raw[6..10].copy_from_slice(&body[7..11]);
        raw[10..14].copy_from_slice(&body[12..16]);
        raw[14..18].copy_from_slice(&body[17..21]);
        raw[18..26].copy_from_slice(&body[22..30]);
        raw.make_ascii_uppercase();

        let bytes = BASE32_NOPAD.decode(&raw)?;
        let uuid = Uuid::from_slice(&bytes).map_err(|_| Error::InvalidEncoding)?;

        // The prefix is a verbatim slice of the (UTF-8) input; reject a split char boundary.
        let prefix = core::str::from_utf8(prefix_bytes).map_err(|_| Error::InvalidEncoding)?;
        Ok(Self::from_uuid(uuid, prefix))
    }
}

impl From<Xuid> for Uuid {
    fn from(x: Xuid) -> Uuid {
        x.uuid
    }
}
