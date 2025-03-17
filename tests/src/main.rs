// use libmdbx_bindings::Archive;
use libmdbx_bindings::DbTx;
use libmdbx_bindings::DbTxMut;
use libmdbx_bindings::TableSet;
use libmdbx_bindings::table_value_codecs_with_zc;
use libmdbx_bindings::{LibmdbxProvider, db_table, tables};
// use serde::Deserialize;
// use serde::Serialize;

fn main() {
    let db = LibmdbxProvider::init_db::<_, MyTables>("./db-test").unwrap();

    db.write::<_, _, MyTables>(|txn| {
        txn.put::<EmptyStrategyTable>(
            100,
            Thing {
                hi: "to me".to_string(),
                this: 1220.0,
            },
        )
    })
    .unwrap()
    .unwrap();

    let out = db
        .read::<_, _, MyTables>(|txn| txn.get::<EmptyStrategyTable>(100))
        .unwrap()
        .unwrap()
        .unwrap();
    println!("OUT: {out:?}");
    // rkyv::to_bytes
    // ArchivedThing::d

    // Thing::default().to_bytes();
}

tables!(MyTables, 1, [EmptyStrategyTable]);

db_table!((EmptyStrategyTable) | u8, Thing);

#[derive(
    Default,
    Debug,
    // libmdbx_bindings::Archive,
    // libmdbx_bindings::Serialize_serde,
    // libmdbx_bindings::Deserialize_serde,
    // libmdbx_bindings::Serialize_rkyv,
    // libmdbx_bindings::Deserialize_rkyv,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
// #[libmdbx_bindings::derive_libmdbx_value]
// #[derive(Debug)]
pub struct Thing {
    hi: String,
    this: f64,
}

table_value_codecs_with_zc!(Thing);
