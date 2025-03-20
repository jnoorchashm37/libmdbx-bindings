pub(crate) mod implementation;
pub(crate) mod provider;
#[macro_use]
pub(crate) mod tables;
pub(crate) mod traits;
#[macro_use]
pub(crate) mod codecs;

pub use bytes::BufMut;
pub use implementation::LibmdbxTx;
pub use libmdbx_native::{RO, RW};
pub use provider::LibmdbxProvider;
pub use reth_db::table::Table;
pub use reth_db::table::{Compress, Decompress};
pub use reth_db::{
    DatabaseError, TableType,
    cursor::{DbCursorRO, DbCursorRW},
    transaction::{DbTx, DbTxMut},
};

pub use traits::*;

pub use paste::paste;

#[cfg(feature = "derive")]
pub use re_exports::*;
#[cfg(feature = "derive")]
mod re_exports {
    pub use rkyv::{
        Archive,
        Infallible, //Serialize as Serialize_rkyv, Deserialize as Deserialize_rkyv,
        archived_root,
        to_bytes,
    };

    pub use alloy_primitives::bytes::BufMut as AlloyBytesMut;
    pub use alloy_rlp::Result as RlpResult;
    pub use alloy_rlp::{Decodable, Encodable};
    pub use libmdbx_bindings_derive::derive_libmdbx_value;

    pub use reth_db_api::table::{Decode, Encode};
    pub use serde::{Deserialize as Deserialize_serde, Serialize as Serialize_serde};
    pub use zstd::{decode_all, encode_all};
}

#[cfg(feature = "derive")]
pub mod re_export_serde {
    pub use serde::{Deserialize, Serialize};
}

#[cfg(feature = "derive")]
pub mod re_export_rkyv {
    pub use rkyv::{Deserialize, Serialize};
}
