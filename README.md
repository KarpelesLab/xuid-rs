# xuid-rs

[![Crates.io](https://img.shields.io/crates/v/xuid-rs.svg)](https://crates.io/crates/xuid-rs)
[![Docs.rs](https://docs.rs/xuid-rs/badge.svg)](https://docs.rs/xuid-rs)

XUID (eXtended Unique IDentifier) enhances UUIDs with a short **type prefix** while
keeping the standard 128-bit UUID size. XUIDs are encoded in **base32** rather than
base16, so they never exceed 36 characters and are more human-readable.

This is a Rust port of the Go library [`KarpelesLab/xuid`](https://github.com/KarpelesLab/xuid).
The string format is **byte-for-byte compatible** — an XUID produced by either
implementation parses cleanly in the other (verified by cross-compat test vectors).

> The crate is published as `xuid-rs` (the name `xuid` was already taken on crates.io),
> but the library is imported as `xuid`.

## Features

- Same number of bits as a standard UUID
- Descriptive type prefixes (up to 5 characters)
- Compact, readable base32 strings (≤ 36 chars), case-insensitive on parse
- Time-ordered **UUIDv7** by default (sortable, index-friendly); UUIDv4 available
- Deterministic IDs from a key (UUIDv5 / SHA-1)
- Trivial conversion to/from standard UUIDs
- Optional `serde` support (serializes as a string)

## Installation

```toml
[dependencies]
xuid-rs = "0.1"
# with JSON / serde support:
# xuid-rs = { version = "0.1", features = ["serde"] }
```

## Usage

```rust
use xuid::Xuid;

// Time-ordered (UUIDv7) by default.
let id = Xuid::new("user");
println!("{id}"); // e.g. user-h4nu2n-zu3f-dmnn-kguv-6f643nei

// Parse — also accepts a plain UUID string (empty prefix).
let parsed: Xuid = "user-h4nu2n-zu3f-dmnn-kguv-6f643nei".parse().unwrap();
assert_eq!(parsed.prefix(), "user");
assert_eq!(parsed.to_uuid_string(), "3f1b4d37-34d9-46c6-b546-a57c5f736d22");

// Type-safe parse: error if the prefix doesn't match.
let _ = Xuid::parse_prefix("user-h4nu2n-zu3f-dmnn-kguv-6f643nei", "user").unwrap();

// Build from an existing UUID, or generate a fully random (v4) one.
let from_v4 = Xuid::new_random("img");

// Deterministic: same key (+ prefix) always yields the same XUID.
let a = Xuid::from_key_prefix("specific-resource-name", "res");
let b = Xuid::from_key_prefix("specific-resource-name", "res");
assert_eq!(a, b);
```

## Format

```
prefix-aaaaaa-aaaa-aaaa-aaaa-aaaaaaaa
```

- `prefix` — 1–5 characters identifying the entity type (omitted entirely when empty)
- the rest is the base32-encoded UUID, hyphenated in 6-4-4-4-8 groups
- output is always lowercase

Examples:

| UUID | Prefix | XUID |
|------|--------|------|
| `3f1b4d37-34d9-46c6-b546-a57c5f736d22` | `shell` | `shell-h4nu2n-zu3f-dmnn-kguv-6f643nei` |
| `00000000-0000-0000-0000-000000000000` | `null`  | `null-aaaaaa-aaaa-aaaa-aaaa-aaaaaaaa` |

### Compatibility notes (matching the Go reference)

- On rendering, the prefix is clipped to its first 5 characters and lowercased; the
  full prefix is still retained in the value. As a result, two XUIDs whose prefixes
  differ only in case (or beyond the 5th char) compare unequal but render identically.
- `from_key` always uses the `utref` prefix and a fixed reference namespace.

## License

BSD 3-Clause. See [LICENSE](LICENSE).
