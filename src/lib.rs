pub(crate) mod implementation;
pub(crate) mod provider;
#[macro_use]
pub(crate) mod tables;
pub(crate) mod traits;
#[macro_use]
pub(crate) mod codecs;

pub use bytes::BufMut;
pub use codecs::Wrapper;
pub use implementation::LibmdbxTx;
pub use libmdbx_native::{RO, RW};
pub use provider::LibmdbxProvider;
pub use reth_db::table::Table;
pub use reth_db::table::{Compress, Decompress};
pub use reth_db::{
    DatabaseError, TableType,
    transaction::{DbTx, DbTxMut},
};

// pub use traits::{TableDet, TableSet, WrapEncodable, WrapDecodable};
pub use traits::*;

pub use re_exports::*;
mod re_exports {
    pub use rkyv::{
        Archive, Deserialize as Deserialize_rkyv, Serialize as Serialize_rkyv, archived_root,
    };

    pub use reth_db_api::table::{Decode, Encode};

    pub use alloy_primitives::bytes::BufMut as AlloyBytesMut;
    pub use alloy_rlp::Result as RlpResult;
    pub use alloy_rlp::{Decodable, Encodable};
    pub use paste::paste;
    pub use serde::{Deserialize as Deserialize_serde, Serialize as Serialize_serde};
    pub use zstd::{decode_all, encode_all};
}
