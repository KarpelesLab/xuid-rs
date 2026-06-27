#![cfg(feature = "rusqlite")]

use rusqlite::Connection;
use xuid::Xuid;

#[test]
fn rusqlite_round_trip() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE items (id TEXT NOT NULL)", [])
        .unwrap();

    let id = Xuid::from_key_prefix("specific-resource-name", "res");
    conn.execute("INSERT INTO items (id) VALUES (?1)", [&id])
        .unwrap();

    // Stored as the textual XUID form.
    let stored: String = conn
        .query_row("SELECT id FROM items", [], |r| r.get(0))
        .unwrap();
    assert_eq!(stored, "res-rgdcy4-j4rj-lejb-fsih-ep4vvn44");

    // Reads back into an Xuid via FromSql.
    let got: Xuid = conn
        .query_row("SELECT id FROM items", [], |r| r.get(0))
        .unwrap();
    assert_eq!(got, id);

    // Invalid text surfaces as an error rather than a panic.
    conn.execute("INSERT INTO items (id) VALUES ('not-an-xuid')", [])
        .unwrap();
    let err: rusqlite::Result<Xuid> =
        conn.query_row("SELECT id FROM items WHERE id = 'not-an-xuid'", [], |r| {
            r.get(0)
        });
    assert!(err.is_err());
}
