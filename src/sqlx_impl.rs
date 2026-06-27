//! [`sqlx`] support: an XUID is stored as its textual representation, generic over any
//! backend whose `String`/`str` types implement the relevant sqlx traits (Postgres,
//! MySQL, SQLite, ...). This mirrors the Go reference, which persists the XUID string.

use crate::Xuid;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::{Database, Decode, Encode, Type};

impl<DB: Database> Type<DB> for Xuid
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

impl<'q, DB: Database> Encode<'q, DB> for Xuid
where
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<'q, DB>>::encode(self.to_string(), buf)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for Xuid
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<'r, DB>>::decode(value)?;
        Ok(s.parse()?)
    }
}
