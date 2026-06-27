//! [`rusqlite`] support: an XUID is stored as `TEXT` (its string representation) and
//! parsed back on read, mirroring the Go reference's `sql.Scanner`/`driver.Valuer`.

use crate::Xuid;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value, ValueRef};

impl ToSql for Xuid {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(Value::Text(self.to_string())))
    }
}

impl FromSql for Xuid {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value
            .as_str()?
            .parse()
            .map_err(|e| FromSqlError::Other(Box::new(e)))
    }
}
