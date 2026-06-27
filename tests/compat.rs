//! Cross-compatibility tests. All expected values were produced by the reference Go
//! implementation (KarpelesLab/xuid) and must remain byte-for-byte identical.

use uuid::Uuid;
use xuid::{Error, Xuid};

/// (uuid, prefix) -> rendered XUID, verified against the Go lib.
const VECTORS: &[(&str, &str, &str)] = &[
    (
        "3f1b4d37-34d9-46c6-b546-a57c5f736d22",
        "user",
        "user-h4nu2n-zu3f-dmnn-kguv-6f643nei",
    ),
    (
        "00000000-0000-0000-0000-000000000000",
        "user",
        "user-aaaaaa-aaaa-aaaa-aaaa-aaaaaaaa",
    ),
    (
        "ffffffff-ffff-ffff-ffff-ffffffffffff",
        "user",
        "user-777777-7777-7777-7777-77777774",
    ),
    (
        "01234567-89ab-cdef-0123-456789abcdef",
        "user",
        "user-aerukz-4jvp-g66a-jdiv-tytk6n54",
    ),
    (
        "3f1b4d37-34d9-46c6-b546-a57c5f736d22",
        "",
        "h4nu2n-zu3f-dmnn-kguv-6f643nei",
    ),
];

#[test]
fn renders_like_go() {
    for (uuid, prefix, want) in VECTORS {
        let x = Xuid::from_uuid(Uuid::parse_str(uuid).unwrap(), *prefix);
        assert_eq!(x.to_string(), *want, "uuid={uuid} prefix={prefix}");
    }
}

#[test]
fn round_trips() {
    for (uuid, prefix, rendered) in VECTORS {
        let parsed: Xuid = rendered.parse().unwrap();
        assert_eq!(parsed.prefix(), *prefix);
        assert_eq!(parsed.to_uuid_string(), *uuid);
        // and back to the same string
        assert_eq!(parsed.to_string(), *rendered);
    }
}

#[test]
fn prefix_is_truncated_to_five_chars_on_render() {
    let x = Xuid::from_uuid(
        Uuid::parse_str("3f1b4d37-34d9-46c6-b546-a57c5f736d22").unwrap(),
        "toolong",
    );
    assert_eq!(x.to_string(), "toolo-h4nu2n-zu3f-dmnn-kguv-6f643nei");
    // ...but the full prefix is preserved in the struct.
    assert_eq!(x.prefix(), "toolong");
}

#[test]
fn prefix_lowercased_on_render_but_stored_verbatim() {
    let x = Xuid::from_uuid(
        Uuid::parse_str("3f1b4d37-34d9-46c6-b546-a57c5f736d22").unwrap(),
        "USER",
    );
    assert_eq!(x.to_string(), "user-h4nu2n-zu3f-dmnn-kguv-6f643nei");
    assert_eq!(x.prefix(), "USER");
}

#[test]
fn from_key_is_deterministic_and_matches_go() {
    let x = Xuid::from_key("specific-resource-name");
    assert_eq!(x.prefix(), "utref");
    assert_eq!(x.to_uuid_string(), "d7d9e7bc-25d6-5e25-9976-b5ec20bcd4e6");
    assert_eq!(x.to_string(), "utref-27m6pp-bf2z-pclg-lwwx-wcbpgu4y");
    assert_eq!(x, Xuid::from_key("specific-resource-name"));
}

#[test]
fn from_key_prefix_matches_go() {
    let x = Xuid::from_key_prefix("specific-resource-name", "res");
    assert_eq!(x.to_uuid_string(), "89862c71-3c8a-5644-84b2-41c8fe56ade7");
    assert_eq!(x.to_string(), "res-rgdcy4-j4rj-lejb-fsih-ep4vvn44");
}

#[test]
fn parses_plain_uuid_as_fallback() {
    let x: Xuid = "3f1b4d37-34d9-46c6-b546-a57c5f736d22".parse().unwrap();
    assert_eq!(x.prefix(), "");
    assert_eq!(x.to_uuid_string(), "3f1b4d37-34d9-46c6-b546-a57c5f736d22");
}

#[test]
fn parse_prefix_enforces_match() {
    let ok = Xuid::parse_prefix("user-h4nu2n-zu3f-dmnn-kguv-6f643nei", "user");
    assert!(ok.is_ok());

    let err = Xuid::parse_prefix("user-h4nu2n-zu3f-dmnn-kguv-6f643nei", "doc");
    assert!(matches!(err, Err(Error::BadPrefix { .. })));
}

#[test]
fn rejects_garbage() {
    assert!("not a uuid at all".parse::<Xuid>().is_err());
}

#[test]
fn does_not_panic_on_multibyte_input() {
    // Strings whose byte length lands in the XUID-shaped range but which contain
    // multibyte chars at structural offsets must error, never panic.
    for s in [
        "héllo-h4nu2n-zu3f-dmnn-kguv-6f643nei",
        "日本語日本語日本語日",
    ] {
        let _ = s.parse::<Xuid>(); // must not panic
    }
}

#[test]
fn new_v7_is_sortable_by_time() {
    let a = Xuid::new("ev");
    let b = Xuid::new("ev");
    // UUIDv7 embeds a timestamp; b was created no earlier than a.
    assert!(b.uuid() >= a.uuid());
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trip() {
    let x = Xuid::from_uuid(
        Uuid::parse_str("3f1b4d37-34d9-46c6-b546-a57c5f736d22").unwrap(),
        "user",
    );
    let json = serde_json::to_string(&x).unwrap();
    assert_eq!(json, "\"user-h4nu2n-zu3f-dmnn-kguv-6f643nei\"");
    let back: Xuid = serde_json::from_str(&json).unwrap();
    assert_eq!(back, x);
}
